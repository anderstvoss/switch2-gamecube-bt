//! Validated controller connection state.

use std::fmt;

/// Observable lifecycle state for one controller.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConnectionState {
    /// No observation has been made.
    #[default]
    Unknown,
    /// The operating system discovered the controller.
    Discovered,
    /// A pairing request is active.
    Pairing,
    /// The operating system reports a bond.
    Paired,
    /// A connection request is active.
    Connecting,
    /// Bluetooth or USB transport is connected.
    Connected,
    /// A usable HID input endpoint is available.
    HidReady,
    /// The controller is no longer connected.
    Disconnected,
    /// The current operation failed.
    Error,
}

impl ConnectionState {
    /// Validates and returns the requested next state.
    ///
    /// # Errors
    ///
    /// Returns [`StateTransitionError`] when the requested lifecycle edge is
    /// not permitted.
    pub fn transition(self, next: Self) -> Result<Self, StateTransitionError> {
        let valid = self == next
            || matches!(
                (self, next),
                (
                    Self::Unknown | Self::Error,
                    Self::Discovered | Self::Disconnected
                ) | (
                    Self::Discovered,
                    Self::Pairing | Self::Paired | Self::Connecting | Self::Disconnected
                ) | (Self::Pairing, Self::Paired | Self::Discovered)
                    | (
                        Self::Paired,
                        Self::Connecting | Self::Disconnected | Self::Pairing
                    )
                    | (
                        Self::Connecting,
                        Self::Connected | Self::Paired | Self::Disconnected
                    )
                    | (Self::Connected, Self::HidReady | Self::Disconnected)
                    | (Self::HidReady, Self::Connected | Self::Disconnected)
                    | (
                        Self::Disconnected,
                        Self::Discovered | Self::Connecting | Self::Connected
                    )
                    | (_, Self::Error)
            );
        valid.then_some(next).ok_or(StateTransitionError {
            from: self,
            to: next,
        })
    }
}

/// An invalid lifecycle transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StateTransitionError {
    /// Current state.
    pub from: ConnectionState,
    /// Rejected next state.
    pub to: ConnectionState,
}

impl fmt::Display for StateTransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid transition from {:?} to {:?}",
            self.from, self.to
        )
    }
}

impl std::error::Error for StateTransitionError {}

#[cfg(test)]
mod tests {
    use super::ConnectionState;

    #[test]
    fn accepts_normal_pairing_path() {
        let mut state = ConnectionState::Unknown;
        for next in [
            ConnectionState::Discovered,
            ConnectionState::Pairing,
            ConnectionState::Paired,
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::HidReady,
        ] {
            state = state.transition(next).expect("valid transition");
        }
    }

    #[test]
    fn rejects_skipping_discovery_and_transport() {
        assert!(
            ConnectionState::Unknown
                .transition(ConnectionState::HidReady)
                .is_err()
        );
        assert!(
            ConnectionState::Paired
                .transition(ConnectionState::HidReady)
                .is_err()
        );
    }
}
