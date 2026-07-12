//! Native Windows platform adapters.

mod bulk;
mod usb;

pub use bulk::{BulkEndpointLayout, inspect_bulk_endpoints};
pub use usb::{
    UsbDescriptorObservation, UsbHidInterface, UsbInputObservation, enumerate_usb_hid,
    inspect_usb_descriptor, observe_usb_input,
};
