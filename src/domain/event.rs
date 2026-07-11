//! Versioned events shared by CLI, service, and future GUI clients.

use super::{ConnectionState, ControllerId, UserSafeError};
use crate::protocol::InputFrame;

/// Current event schema version.
pub const DOMAIN_EVENT_SCHEMA_VERSION: u16 = 1;

/// A portable application event.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum DomainEvent {
    /// A validated state change.
    StateChanged {
        /// Controller affected by the transition.
        controller: ControllerId,
        /// New lifecycle state.
        state: ConnectionState,
    },
    /// A decoded input frame.
    Input(InputFrame),
    /// A user-safe failure.
    Error(UserSafeError),
}

impl DomainEvent {
    /// Returns the event schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        DOMAIN_EVENT_SCHEMA_VERSION
    }
}
