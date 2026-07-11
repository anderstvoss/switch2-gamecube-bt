//! Read-only Windows HID inventory and bounded observation.

use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use hidapi::{BusType, HidApi};
use sha2::{Digest, Sha256};

use crate::domain::{ErrorCategory, UserSafeError};

const MAX_LABEL_LENGTH: usize = 128;

/// Sanitized metadata for one Windows HID interface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsbHidInterface {
    /// USB vendor identifier.
    pub vendor_id: u16,
    /// USB product identifier.
    pub product_id: u16,
    /// HID usage page reported by Windows.
    pub usage_page: u16,
    /// HID usage reported by Windows.
    pub usage: u16,
    /// USB interface number, or a negative value when unavailable.
    pub interface_number: i32,
    /// Sanitized product label, when provided.
    pub product_label: Option<String>,
    /// Sanitized manufacturer label, when provided.
    pub manufacturer_label: Option<String>,
    /// Transport classification reported by the HID library.
    pub bus_type: &'static str,
}

/// Sanitized fingerprint of one HID report descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsbDescriptorObservation {
    /// Descriptor length in bytes.
    pub length: usize,
    /// Lowercase SHA-256 digest of the descriptor bytes.
    pub sha256: String,
}

/// Aggregated metadata for input reports with the same ID and length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UsbInputObservation {
    /// First report byte, treated as the report identifier.
    pub report_id: u8,
    /// Number of bytes returned by Windows.
    pub length: usize,
    /// Number of reports observed in this bucket.
    pub count: usize,
}

/// Enumerates matching HID interfaces without opening a device handle.
///
/// Serial numbers and platform device paths are intentionally never copied
/// into the returned model.
///
/// # Errors
///
/// Returns a privacy-safe platform error when Windows HID enumeration fails.
pub fn enumerate_usb_hid(
    vendor_id: u16,
    product_id: Option<u16>,
) -> Result<Vec<UsbHidInterface>, UserSafeError> {
    let api = HidApi::new().map_err(|_| {
        UserSafeError::new(
            ErrorCategory::Platform,
            "Windows HID inventory initialization failed",
        )
    })?;

    let mut interfaces = api
        .device_list()
        .filter(|device| {
            device.vendor_id() == vendor_id
                && product_id.is_none_or(|expected| device.product_id() == expected)
        })
        .map(|device| UsbHidInterface {
            vendor_id: device.vendor_id(),
            product_id: device.product_id(),
            usage_page: device.usage_page(),
            usage: device.usage(),
            interface_number: device.interface_number(),
            product_label: sanitize_label(device.product_string()),
            manufacturer_label: sanitize_label(device.manufacturer_string()),
            bus_type: bus_type_name(device.bus_type()),
        })
        .collect::<Vec<_>>();

    interfaces.sort_by_key(|device| {
        (
            device.product_id,
            device.interface_number,
            device.usage_page,
            device.usage,
        )
    });
    Ok(interfaces)
}

/// Opens the matching HID interface and reads only its report descriptor.
///
/// The device handle and raw descriptor remain private to this function. No
/// input, output, or feature report API is called.
///
/// # Errors
///
/// Returns a privacy-safe error if enumeration, opening, or descriptor reading
/// fails, or if the identity does not resolve to exactly one HID interface.
pub fn inspect_usb_descriptor(
    vendor_id: u16,
    product_id: u16,
) -> Result<UsbDescriptorObservation, UserSafeError> {
    let api = HidApi::new().map_err(|_| platform_error("Windows HID initialization failed"))?;
    let device = open_unique_usb_device(&api, vendor_id, product_id)?;
    let mut descriptor = vec![0_u8; hidapi::MAX_REPORT_DESCRIPTOR_SIZE];
    let length = device
        .get_report_descriptor(&mut descriptor)
        .map_err(|_| platform_error("HID report descriptor could not be read"))?;
    descriptor.truncate(length);
    let sha256 = format!("{:x}", Sha256::digest(&descriptor));
    Ok(UsbDescriptorObservation { length, sha256 })
}

/// Reads bounded input-report metadata without returning report contents.
///
/// The function never invokes output or feature-report APIs. Report buffers
/// are aggregated to ID, length, and count before crossing this boundary.
///
/// # Errors
///
/// Returns a privacy-safe error when the interface cannot be uniquely opened or
/// Windows fails to read input.
pub fn observe_usb_input(
    vendor_id: u16,
    product_id: u16,
    duration: Duration,
    limit: usize,
) -> Result<Vec<UsbInputObservation>, UserSafeError> {
    let api = HidApi::new().map_err(|_| platform_error("Windows HID initialization failed"))?;
    let device = open_unique_usb_device(&api, vendor_id, product_id)?;
    let deadline = Instant::now() + duration;
    let mut report = vec![0_u8; crate::protocol::MAX_REPORT_SIZE];
    let mut buckets = BTreeMap::<(u8, usize), usize>::new();
    let mut reports_read = 0_usize;

    while reports_read < limit && Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let timeout_ms = remaining.as_millis().clamp(1, 100) as i32;
        let length = device
            .read_timeout(&mut report, timeout_ms)
            .map_err(|_| platform_error("USB HID input observation failed"))?;
        if length == 0 {
            continue;
        }
        record_report(&mut buckets, &report[..length]);
        report[..length].fill(0);
        reports_read += 1;
    }

    Ok(buckets
        .into_iter()
        .map(|((report_id, length), count)| UsbInputObservation {
            report_id,
            length,
            count,
        })
        .collect())
}

fn record_report(buckets: &mut BTreeMap<(u8, usize), usize>, report: &[u8]) {
    if let Some(report_id) = report.first() {
        *buckets.entry((*report_id, report.len())).or_default() += 1;
    }
}

fn open_unique_usb_device(
    api: &HidApi,
    vendor_id: u16,
    product_id: u16,
) -> Result<hidapi::HidDevice, UserSafeError> {
    let mut matches = api.device_list().filter(|device| {
        device.vendor_id() == vendor_id
            && device.product_id() == product_id
            && matches!(device.bus_type(), BusType::Usb)
    });
    let info = matches
        .next()
        .ok_or_else(|| platform_error("matching USB HID interface was not found"))?;
    if matches.next().is_some() {
        return Err(UserSafeError::new(
            ErrorCategory::InvalidData,
            "multiple matching USB HID interfaces require explicit selection",
        ));
    }
    info.open_device(api)
        .map_err(|_| platform_error("USB HID interface could not be opened for inspection"))
}

fn platform_error(message: &'static str) -> UserSafeError {
    UserSafeError::new(ErrorCategory::Platform, message)
}

fn sanitize_label(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() || value.chars().any(char::is_control) {
        return None;
    }
    Some(value.chars().take(MAX_LABEL_LENGTH).collect())
}

const fn bus_type_name(bus_type: BusType) -> &'static str {
    match bus_type {
        BusType::Usb => "usb",
        BusType::Bluetooth => "bluetooth",
        BusType::I2c => "i2c",
        BusType::Spi => "spi",
        BusType::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{record_report, sanitize_label};

    #[test]
    fn labels_are_bounded_and_reject_control_data() {
        assert_eq!(sanitize_label(Some(" Nintendo ")), Some("Nintendo".into()));
        assert_eq!(sanitize_label(Some("bad\nlabel")), None);
        assert_eq!(
            sanitize_label(Some(&"x".repeat(256)))
                .expect("printable label")
                .len(),
            128
        );
    }

    #[test]
    fn report_metadata_is_aggregated_without_contents() {
        let mut buckets = BTreeMap::new();
        record_report(&mut buckets, &[0x05, 0xaa, 0xbb]);
        record_report(&mut buckets, &[0x05, 0x00, 0x01]);
        record_report(&mut buckets, &[]);
        assert_eq!(buckets, [((0x05, 3), 2)].into());
    }
}
