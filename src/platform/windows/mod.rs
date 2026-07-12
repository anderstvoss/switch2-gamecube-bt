//! Native Windows platform adapters.

mod bluetooth;
mod bulk;
mod pairtool;
mod usb;

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
pub use pairtool::{
    PairToolDiscoveryObservation, PairToolStatus, discover_with_pairtool, inspect_pairtool,
};
pub use usb::{
    UsbDecodedInputObservation, UsbDescriptorObservation, UsbHidInterface, UsbInputObservation,
    enumerate_usb_hid, inspect_usb_descriptor, observe_calibrated_usb_input,
    observe_decoded_usb_input, observe_usb_input,
};
