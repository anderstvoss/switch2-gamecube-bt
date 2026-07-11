//! Opaque platform identity types.

const MAX_IDENTIFIER_LEN: usize = 256;

/// An error returned when a platform identifier is unsafe or malformed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierError {
    /// The identifier was empty or contained only whitespace.
    Empty,
    /// The identifier exceeded the bounded domain representation.
    TooLong,
    /// The identifier contained a control character.
    ControlCharacter,
}

macro_rules! opaque_identifier {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates an opaque identifier after validating its safe bounds.
            ///
            /// # Errors
            ///
            /// Returns [`IdentifierError`] when the value is empty, oversized,
            /// or contains control characters.
            pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(IdentifierError::Empty);
                }
                if value.len() > MAX_IDENTIFIER_LEN {
                    return Err(IdentifierError::TooLong);
                }
                if value.chars().any(char::is_control) {
                    return Err(IdentifierError::ControlCharacter);
                }
                Ok(Self(value))
            }

            /// Returns the platform-owned opaque value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

opaque_identifier!(AdapterId, "An opaque Bluetooth adapter identifier.");
opaque_identifier!(ControllerId, "An opaque controller identifier.");

#[cfg(test)]
mod tests {
    use super::{AdapterId, ControllerId, IdentifierError, MAX_IDENTIFIER_LEN};

    #[test]
    fn accepts_bounded_opaque_identifiers() {
        let id = AdapterId::new("platform-adapter-id").expect("valid identifier");
        assert_eq!(id.as_str(), "platform-adapter-id");
    }

    #[test]
    fn rejects_unsafe_identifiers() {
        assert_eq!(ControllerId::new("  "), Err(IdentifierError::Empty));
        assert_eq!(
            ControllerId::new("device\nsecret"),
            Err(IdentifierError::ControlCharacter)
        );
        assert_eq!(
            ControllerId::new("x".repeat(MAX_IDENTIFIER_LEN + 1)),
            Err(IdentifierError::TooLong)
        );
    }
}
