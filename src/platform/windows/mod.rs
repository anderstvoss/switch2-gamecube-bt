//! Native Windows platform adapters.

mod usb;

pub use usb::{
    UsbDescriptorObservation, UsbHidInterface, UsbInputObservation, enumerate_usb_hid,
    inspect_usb_descriptor, observe_usb_input,
};
