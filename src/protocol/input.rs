//! Normalized, full-fidelity controller input.

use std::collections::{BTreeMap, BTreeSet};

/// Evidence-backed controller buttons.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Button {
    /// Primary A button.
    A,
    /// Primary B button.
    B,
    /// Primary X button.
    X,
    /// Primary Y button.
    Y,
    /// Start/Pause button.
    Start,
    /// Home button.
    Home,
    /// Capture button.
    Capture,
    /// `GameChat` C button.
    Chat,
    /// D-pad up direction.
    DpadUp,
    /// D-pad down direction.
    DpadDown,
    /// D-pad left direction.
    DpadLeft,
    /// D-pad right direction.
    DpadRight,
    /// L shoulder control.
    L,
    /// R shoulder control.
    R,
    /// Z shoulder button.
    Z,
    /// ZL shoulder button.
    ZL,
}

/// Normalized controller axes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Axis {
    /// Main stick horizontal axis.
    LeftX,
    /// Main stick vertical axis.
    LeftY,
    /// C-stick horizontal axis.
    CStickX,
    /// C-stick vertical axis.
    CStickY,
    /// Analog left-trigger axis.
    LeftTrigger,
    /// Analog right-trigger axis.
    RightTrigger,
}

/// A bounded battery percentage when exposed by the controller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryLevel(u8);

impl BatteryLevel {
    /// Creates a battery percentage in the inclusive range 0 through 100.
    #[must_use]
    pub const fn new(percent: u8) -> Option<Self> {
        if percent <= 100 {
            Some(Self(percent))
        } else {
            None
        }
    }

    /// Returns the percentage.
    #[must_use]
    pub const fn percent(self) -> u8 {
        self.0
    }
}

/// One normalized accelerometer and gyroscope sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionSample {
    /// Acceleration in meters per second squared.
    pub acceleration: [f32; 3],
    /// Angular velocity in radians per second.
    pub angular_velocity: [f32; 3],
}

/// One decoded controller input frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputFrame {
    /// Currently pressed buttons.
    pub buttons: BTreeSet<Button>,
    /// Signed normalized axes in the inclusive range -32767 through 32767.
    pub axes: BTreeMap<Axis, i16>,
    /// Battery level when available.
    pub battery: Option<BatteryLevel>,
    /// Motion samples retained in transport order.
    pub motion: Vec<MotionSample>,
}
