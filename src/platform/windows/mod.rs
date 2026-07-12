//! Native Windows platform adapters.

mod bulk;
mod usb;

pub use bulk::{
    BulkEndpointLayout, BulkReportObservation, MinimalInputProbeObservation,
    inspect_bulk_endpoints, run_described_input_probe, run_minimal_input_probe,
    run_report5_input_probe, run_sdl_reference_input_probe,
};
pub use usb::{
    UsbDecodedInputObservation, UsbDescriptorObservation, UsbHidInterface, UsbInputObservation,
    enumerate_usb_hid, inspect_usb_descriptor, observe_decoded_usb_input, observe_usb_input,
};
