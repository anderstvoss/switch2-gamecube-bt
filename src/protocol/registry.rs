//! Evidence-backed controller decoder registry.

use crate::domain::UserSafeError;

use super::{InputFrame, RawReport};

/// Sanitized identity fields used to select a decoder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerDescriptor {
    /// Optional USB vendor identifier.
    pub vendor_id: Option<u16>,
    /// Optional USB product identifier.
    pub product_id: Option<u16>,
    /// Sanitized operating-system display name.
    pub display_name: Option<String>,
}

/// A model-specific, transport-independent report decoder.
pub trait ReportDecoder: Send + Sync {
    /// Stable decoder identifier.
    fn id(&self) -> &'static str;

    /// Returns whether evidence supports using this decoder for the descriptor.
    fn supports(&self, descriptor: &ControllerDescriptor) -> bool;

    /// Decodes one bounded report without discarding unknown reports globally.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe error when a recognized report is malformed.
    fn decode(&self, report: &RawReport) -> Result<Option<InputFrame>, UserSafeError>;
}

/// Ordered model decoder registry.
#[derive(Default)]
pub struct ControllerRegistry {
    decoders: Vec<Box<dyn ReportDecoder>>,
}

impl ControllerRegistry {
    /// Adds a decoder after rejecting duplicate stable identifiers.
    ///
    /// # Errors
    ///
    /// Returns an error if a decoder with the same stable identifier is
    /// already registered.
    pub fn register(&mut self, decoder: Box<dyn ReportDecoder>) -> Result<(), &'static str> {
        if self
            .decoders
            .iter()
            .any(|candidate| candidate.id() == decoder.id())
        {
            return Err("duplicate decoder identifier");
        }
        self.decoders.push(decoder);
        Ok(())
    }

    /// Selects the first evidence-backed decoder for a descriptor.
    #[must_use]
    pub fn find(&self, descriptor: &ControllerDescriptor) -> Option<&dyn ReportDecoder> {
        self.decoders
            .iter()
            .find(|decoder| decoder.supports(descriptor))
            .map(AsRef::as_ref)
    }
}
