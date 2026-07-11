//! Operating-system-independent controller domain types.

mod capability;
mod error;
mod event;
mod identity;
mod operation;
mod state;

pub use capability::{Capability, CapabilitySet};
pub use error::{ErrorCategory, UserSafeError};
pub use event::{DOMAIN_EVENT_SCHEMA_VERSION, DomainEvent};
pub use identity::{AdapterId, ControllerId, IdentifierError};
pub use operation::{CancellationToken, Deadline};
pub use state::{ConnectionState, StateTransitionError};
