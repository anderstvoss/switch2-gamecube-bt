//! Explicitly gated controller outputs.

/// A verified volatile output operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerifiedOutput {
    /// Evidence-backed transient rumble intensity.
    Rumble {
        /// Low-frequency motor intensity.
        low: u8,
        /// High-frequency motor intensity.
        high: u8,
    },
    /// Evidence-backed transient player-light mask.
    PlayerLights(u8),
}

/// Output policy request.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputRequest {
    /// Reject all controller output. This is the default policy.
    #[default]
    Deny,
    /// Submit an operation previously verified as volatile for this model.
    VerifiedVolatile(VerifiedOutput),
}

#[cfg(test)]
mod tests {
    use super::OutputRequest;

    #[test]
    fn output_is_denied_by_default() {
        assert_eq!(OutputRequest::default(), OutputRequest::Deny);
    }
}
