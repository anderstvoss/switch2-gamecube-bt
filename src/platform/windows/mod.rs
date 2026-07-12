//! Native Windows platform adapters.

mod ble;
mod bluetooth;
mod bulk;
mod package;
mod pairtool;
mod usb;

pub use ble::{
    BleAdapterCapabilities, BleAdvertisementObservation, BleDeviceObservation,
    BleDeviceScanObservation, BleScanObservation, inspect_ble_adapter, scan_ble_advertisements,
    scan_unpaired_ble_devices,
};
pub use bluetooth::{
    BluetoothDeviceObservation, BluetoothInventoryObservation, BluetoothScanObservation,
    enumerate_bluetooth, scan_unpaired_bluetooth,
};
pub use bulk::{
    BulkEndpointLayout, BulkReportObservation, CalibrationObservation,
    MinimalInputProbeObservation, inspect_bulk_endpoints, read_calibration,
    run_described_input_probe, run_minimal_input_probe, run_motion_enable_probe,
    run_report5_input_probe, run_sdl_reference_input_probe,
};
pub use package::has_package_identity;
pub use pairtool::{
    PairToolDiscoveryObservation, PairToolStatus, discover_with_pairtool, inspect_pairtool,
};
pub use usb::{
    UsbDecodedInputObservation, UsbDescriptorObservation, UsbHidInterface, UsbInputObservation,
    enumerate_usb_hid, inspect_usb_descriptor, observe_calibrated_usb_input,
    observe_decoded_usb_input, observe_usb_input,
};
