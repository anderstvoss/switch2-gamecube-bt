//! Transport-independent controller protocol types.

mod input;
mod output;
mod registry;
mod report;

pub use input::{Axis, BatteryLevel, Button, InputFrame, MotionSample};
pub use output::{OutputRequest, VerifiedOutput};
pub use registry::{ControllerDescriptor, ControllerRegistry, ReportDecoder};
pub use report::{MAX_REPORT_SIZE, RawReport, ReportError, Transport};
