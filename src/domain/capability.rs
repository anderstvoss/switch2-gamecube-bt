//! Backend and controller capability declarations.

use std::collections::BTreeSet;

/// A discrete operation supported by a backend or controller.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Capability {
    /// Enumerate adapters.
    AdapterInventory,
    /// Discover nearby controllers.
    Discovery,
    /// Ask the operating system to pair a controller.
    Pairing,
    /// Establish a connection to a known controller.
    Connection,
    /// Inspect HID interfaces.
    HidInventory,
    /// Read bounded input reports.
    InputReports,
    /// Send explicitly allowlisted volatile output reports.
    VerifiedVolatileOutput,
    /// Submit normalized input to a virtual controller.
    VirtualOutput,
}

/// An ordered, duplicate-free set of capabilities.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapabilitySet(BTreeSet<Capability>);

impl CapabilitySet {
    /// Builds a capability set from an iterator.
    #[must_use]
    pub fn from_capabilities(values: impl IntoIterator<Item = Capability>) -> Self {
        Self(values.into_iter().collect())
    }

    /// Returns whether the capability is explicitly supported.
    #[must_use]
    pub fn contains(&self, capability: Capability) -> bool {
        self.0.contains(&capability)
    }

    /// Iterates in stable capability order.
    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.0.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::{Capability, CapabilitySet};

    #[test]
    fn capabilities_are_explicit_and_deduplicated() {
        let capabilities = CapabilitySet::from_capabilities([
            Capability::Discovery,
            Capability::Discovery,
            Capability::InputReports,
        ]);
        assert!(capabilities.contains(Capability::Discovery));
        assert!(!capabilities.contains(Capability::Pairing));
        assert_eq!(capabilities.iter().count(), 2);
    }
}
