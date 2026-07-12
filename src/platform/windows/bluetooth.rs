//! Read-only Windows Bluetooth inventory.

use sha2::{Digest, Sha256};
use windows::{
    Devices::{Bluetooth::BluetoothAdapter, Enumeration::DeviceInformation},
    core::HSTRING,
};

use crate::domain::{ErrorCategory, UserSafeError};

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
        let id = device.Id().map_err(platform_error)?;
        let pairing = device.Pairing().map_err(platform_error)?;
        observations.push(BluetoothDeviceObservation {
            id_digest: digest_identifier(&id),
            name: non_empty_name(device.Name().ok().map(|name| name.to_string_lossy())),
            paired: pairing.IsPaired().map_err(platform_error)?,
            enabled: device.IsEnabled().map_err(platform_error)?,
        });
    }

    Ok(BluetoothInventoryObservation {
        adapter_present: true,
        devices: observations,
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
