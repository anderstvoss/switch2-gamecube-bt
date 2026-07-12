//! Read-only Windows Bluetooth inventory.

use std::{
    collections::BTreeMap,
    sync::mpsc,
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use windows::{
    Devices::{
        Bluetooth::BluetoothAdapter,
        Enumeration::{DeviceInformation, DeviceInformationKind},
    },
    Foundation::TypedEventHandler,
    core::HSTRING,
};
use windows_collections::IIterable;

use crate::domain::{ErrorCategory, UserSafeError};

const CLASSIC_BLUETOOTH_AEP_SELECTOR: &str =
    "System.Devices.Aep.ProtocolId:=\"{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}\"";
const MIN_SCAN_DURATION: Duration = Duration::from_secs(1);
const MAX_SCAN_DURATION: Duration = Duration::from_secs(10);

/// Sanitized Bluetooth device metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BluetoothDeviceObservation {
    /// Stable per-host digest of the Windows device identifier.
    pub id_digest: String,
    /// Windows display name, when provided by the adapter.
    pub name: Option<String>,
    /// Whether Windows currently reports the device as paired.
    pub paired: bool,
    /// Whether Windows currently reports the device as enabled.
    pub enabled: bool,
}

/// Read-only Bluetooth adapter and device inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BluetoothInventoryObservation {
    /// Whether the default Windows Bluetooth adapter was available.
    pub adapter_present: bool,
    /// Devices returned by the Windows Bluetooth device selector.
    pub devices: Vec<BluetoothDeviceObservation>,
}

/// Bounded result of an unpaired Bluetooth discovery scan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BluetoothScanObservation {
    /// Duration for which the Windows watcher was active.
    pub duration: Duration,
    /// Sanitized unpaired devices observed during the scan.
    pub devices: Vec<BluetoothDeviceObservation>,
}

/// Enumerate Bluetooth devices without pairing, connecting, or reading link keys.
///
/// # Errors
///
/// Returns a sanitized platform error if Windows cannot enumerate the selector
/// or read one of the returned device properties.
pub fn enumerate_bluetooth() -> Result<BluetoothInventoryObservation, UserSafeError> {
    let _adapter = BluetoothAdapter::GetDefaultAsync()
        .map_err(platform_error)?
        .get()
        .map_err(platform_error)?;
    let selector = windows::Devices::Bluetooth::BluetoothDevice::GetDeviceSelector()
        .map_err(platform_error)?;
    let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)
        .map_err(platform_error)?
        .get()
        .map_err(platform_error)?;

    let mut observations = Vec::with_capacity(devices.Size().map_err(platform_error)? as usize);
    for index in 0..devices.Size().map_err(platform_error)? {
        let device = devices.GetAt(index).map_err(platform_error)?;
        observations.push(observe_device(&device)?);
    }

    Ok(BluetoothInventoryObservation {
        adapter_present: true,
        devices: observations,
    })
}

/// Watch for nearby unpaired classic-Bluetooth association endpoints.
///
/// This creates a Windows `DeviceWatcher` with the Bluetooth Classic association
/// endpoint selector used by Windows pairing.
/// It does not pair, connect, or access link keys.
///
/// # Errors
///
/// Returns a sanitized error if the duration is outside one to ten seconds or
/// if Windows cannot create, start, or stop the watcher.
pub fn scan_unpaired_bluetooth(
    duration: Duration,
) -> Result<BluetoothScanObservation, UserSafeError> {
    validate_scan_duration(duration)?;

    let selector = HSTRING::from(CLASSIC_BLUETOOTH_AEP_SELECTOR);
    let properties = IIterable::<HSTRING>::from(Vec::new());
    let watcher = DeviceInformation::CreateWatcherWithKindAqsFilterAndAdditionalProperties(
        &selector,
        &properties,
        DeviceInformationKind::AssociationEndpoint,
    )
    .map_err(platform_error)?;
    let (sender, receiver) = mpsc::channel();
    let handler = TypedEventHandler::new(move |_, device| {
        if let Some(device) = &*device
            && let Ok(observation) = observe_device(device)
        {
            let _ = sender.send(observation);
        }
        Ok(())
    });
    let token = watcher.Added(&handler).map_err(platform_error)?;
    watcher.Start().map_err(platform_error)?;

    let deadline = Instant::now() + duration;
    let mut devices = BTreeMap::new();
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        match receiver.recv_timeout(remaining) {
            Ok(device) => {
                devices.insert(device.id_digest.clone(), device);
            }
            Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let stop_result = watcher.Stop().map_err(platform_error);
    let remove_result = watcher.RemoveAdded(token).map_err(platform_error);
    stop_result?;
    remove_result?;
    Ok(BluetoothScanObservation {
        duration,
        devices: devices.into_values().collect(),
    })
}

fn validate_scan_duration(duration: Duration) -> Result<(), UserSafeError> {
    if (MIN_SCAN_DURATION..=MAX_SCAN_DURATION).contains(&duration) {
        Ok(())
    } else {
        Err(UserSafeError::new(
            ErrorCategory::InvalidData,
            "Bluetooth scan duration must be between one and ten seconds",
        ))
    }
}

fn observe_device(device: &DeviceInformation) -> Result<BluetoothDeviceObservation, UserSafeError> {
    let id = device.Id().map_err(platform_error)?;
    let pairing = device.Pairing().map_err(platform_error)?;
    Ok(BluetoothDeviceObservation {
        id_digest: digest_identifier(&id),
        name: non_empty_name(device.Name().ok().map(|name| name.to_string_lossy())),
        paired: pairing.IsPaired().map_err(platform_error)?,
        enabled: device.IsEnabled().map_err(platform_error)?,
    })
}

fn digest_identifier(identifier: &HSTRING) -> String {
    let digest = Sha256::digest(identifier.to_string_lossy().as_bytes());
    let mut result = String::with_capacity(16);
    for byte in &digest[..8] {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn non_empty_name(name: Option<String>) -> Option<String> {
    name.filter(|value| !value.trim().is_empty())
}

fn platform_error(error: impl std::fmt::Display) -> UserSafeError {
    UserSafeError::new(
        ErrorCategory::Platform,
        format!("Bluetooth inventory failed: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::domain::ErrorCategory;

    use super::{
        CLASSIC_BLUETOOTH_AEP_SELECTOR, MAX_SCAN_DURATION, MIN_SCAN_DURATION,
        validate_scan_duration,
    };

    #[test]
    fn scan_duration_is_bounded_for_the_observed_pairing_window() {
        assert!(validate_scan_duration(MIN_SCAN_DURATION).is_ok());
        assert!(validate_scan_duration(MAX_SCAN_DURATION).is_ok());
        assert_eq!(
            validate_scan_duration(Duration::ZERO)
                .expect_err("zero duration must be rejected")
                .category(),
            ErrorCategory::InvalidData
        );
        assert_eq!(
            validate_scan_duration(Duration::from_secs(11))
                .expect_err("overlong duration must be rejected")
                .category(),
            ErrorCategory::InvalidData
        );
    }

    #[test]
    fn scan_uses_the_documented_classic_bluetooth_aep_protocol() {
        assert_eq!(
            CLASSIC_BLUETOOTH_AEP_SELECTOR,
            "System.Devices.Aep.ProtocolId:=\"{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}\""
        );
    }
}
