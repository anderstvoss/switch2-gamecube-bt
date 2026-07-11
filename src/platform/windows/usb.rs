//! Read-only Windows HID inventory.

use hidapi::{BusType, HidApi};

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
    use super::sanitize_label;

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
}
