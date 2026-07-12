//! Read-only Windows Bluetooth Low Energy advertisement discovery.

use std::{
    collections::BTreeMap,
    sync::mpsc,
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use windows::{
    Devices::Bluetooth::{
        Advertisement::{
            BluetoothLEAdvertisementReceivedEventArgs, BluetoothLEAdvertisementWatcher,
            BluetoothLEScanningMode,
        },
        BluetoothAdapter,
    },
    Foundation::TypedEventHandler,
    core::GUID,
};

use crate::domain::{ErrorCategory, UserSafeError};

const SWITCH2_GATT_SERVICE: GUID = GUID::from_u128(0xab7de9be_89fe_49ad_828f_118f09df7fd0);

/// Sanitized BLE advertisement metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BleAdvertisementObservation {
    /// Short digest of the rotating Bluetooth address; the address is not exposed.
    pub identifier_digest: String,
    /// Advertised local name when provided by the controller.
    pub local_name: Option<String>,
    /// Whether the known Switch 2 GATT service was advertised.
    pub switch2_service_advertised: bool,
}

/// Bounded BLE discovery result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BleScanObservation {
    /// Requested scan duration.
    pub duration: Duration,
    /// Deduplicated sanitized advertisements.
    pub advertisements: Vec<BleAdvertisementObservation>,
}

/// Capabilities required for the host to act as a BLE controller client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BleAdapterCapabilities {
    /// Whether Windows reports BLE support.
    pub low_energy_supported: bool,
    /// Whether Windows reports BLE central-role support.
    pub central_role_supported: bool,
}

/// Read the default adapter's BLE capabilities without scanning or connecting.
///
/// # Errors
///
/// Returns a sanitized platform error if Windows cannot obtain the default
/// Bluetooth adapter or its reported capabilities.
pub fn inspect_ble_adapter() -> Result<BleAdapterCapabilities, UserSafeError> {
    let adapter = BluetoothAdapter::GetDefaultAsync()
        .map_err(platform_error)?
        .get()
        .map_err(platform_error)?;
    Ok(BleAdapterCapabilities {
        low_energy_supported: adapter.IsLowEnergySupported().map_err(platform_error)?,
        central_role_supported: adapter.IsCentralRoleSupported().map_err(platform_error)?,
    })
}

/// Actively scan BLE advertisements without connecting, pairing, or writing.
///
/// # Errors
///
/// Returns a sanitized error when Windows cannot start or stop the watcher, or
/// when the requested duration is outside one to ten seconds.
pub fn scan_ble_advertisements(duration: Duration) -> Result<BleScanObservation, UserSafeError> {
    if !(Duration::from_secs(1)..=Duration::from_secs(10)).contains(&duration) {
        return Err(UserSafeError::new(
            ErrorCategory::InvalidData,
            "BLE scan duration must be between one and ten seconds",
        ));
    }
    let watcher = BluetoothLEAdvertisementWatcher::new().map_err(platform_error)?;
    watcher
        .SetScanningMode(BluetoothLEScanningMode::Active)
        .map_err(platform_error)?;
    let (sender, receiver) = mpsc::channel();
    let handler = TypedEventHandler::new(move |_, args| {
        if let Some(args) = &*args
            && let Ok(observation) = observe_advertisement(args)
        {
            let _ = sender.send(observation);
        }
        Ok(())
    });
    let token = watcher.Received(&handler).map_err(platform_error)?;
    watcher.Start().map_err(platform_error)?;
    let deadline = Instant::now() + duration;
    let mut advertisements = BTreeMap::new();
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        match receiver.recv_timeout(remaining) {
            Ok(advertisement) => {
                advertisements.insert(advertisement.identifier_digest.clone(), advertisement);
            }
            Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let stop_result = watcher.Stop().map_err(platform_error);
    let remove_result = watcher.RemoveReceived(token).map_err(platform_error);
    stop_result?;
    remove_result?;
    Ok(BleScanObservation {
        duration,
        advertisements: advertisements.into_values().collect(),
    })
}

fn observe_advertisement(
    args: &BluetoothLEAdvertisementReceivedEventArgs,
) -> Result<BleAdvertisementObservation, UserSafeError> {
    let advertisement = args.Advertisement().map_err(platform_error)?;
    let services = advertisement.ServiceUuids().map_err(platform_error)?;
    let mut switch2_service_advertised = false;
    for index in 0..services.Size().map_err(platform_error)? {
        if services.GetAt(index).map_err(platform_error)? == SWITCH2_GATT_SERVICE {
            switch2_service_advertised = true;
        }
    }
    Ok(BleAdvertisementObservation {
        identifier_digest: digest_address(args.BluetoothAddress().map_err(platform_error)?),
        local_name: advertisement
            .LocalName()
            .ok()
            .map(|name| name.to_string_lossy())
            .filter(|name| !name.trim().is_empty()),
        switch2_service_advertised,
    })
}

fn digest_address(address: u64) -> String {
    let digest = Sha256::digest(address.to_le_bytes());
    let mut result = String::with_capacity(16);
    for byte in &digest[..8] {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn platform_error(error: impl std::fmt::Display) -> UserSafeError {
    UserSafeError::new(
        ErrorCategory::Platform,
        format!("BLE discovery failed: {error}"),
    )
}
