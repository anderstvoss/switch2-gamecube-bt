//! Privacy-safe public errors.

use std::fmt;

/// Stable public error categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorCategory {
    /// The backend cannot perform the operation.
    Unsupported,
    /// The user or operating system denied access.
    PermissionDenied,
    /// The operation exceeded its deadline.
    Timeout,
    /// Cancellation was requested.
    Cancelled,
    /// Pairing did not complete.
    PairingFailed,
    /// Transport connection did not complete.
    ConnectionFailed,
    /// A usable HID endpoint was unavailable.
    HidUnavailable,
    /// Input or output data was invalid.
    InvalidData,
    /// The platform returned an uncategorized failure.
    Platform,
}

/// An error safe for normal logs and user interfaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserSafeError {
    category: ErrorCategory,
    message: String,
}

impl UserSafeError {
    /// Creates an error from an allowlisted category and redacted message.
    #[must_use]
    pub fn new(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: redact_message(&message.into()),
        }
    }

    /// Returns the stable category.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        self.category
    }

    /// Returns the redacted user-facing message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for UserSafeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for UserSafeError {}

fn redact_message(message: &str) -> String {
    let mut result = String::with_capacity(message.len().min(256));
    for token in message.split_whitespace().take(32) {
        if !result.is_empty() {
            result.push(' ');
        }
        if looks_sensitive(token) {
            result.push_str("<redacted>");
        } else {
            result.extend(token.chars().filter(|character| !character.is_control()));
        }
    }
    result.truncate(result.floor_char_boundary(256));
    result
}

fn looks_sensitive(token: &str) -> bool {
    let colon_groups = token.split(':').collect::<Vec<_>>();
    let looks_like_mac = colon_groups.len() == 6
        && colon_groups.iter().all(|group| {
            group.len() == 2 && group.chars().all(|character| character.is_ascii_hexdigit())
        });
    looks_like_mac || token.contains(['\\', '/'])
}

#[cfg(test)]
mod tests {
    use super::{ErrorCategory, UserSafeError};

    #[test]
    fn redacts_addresses_and_paths() {
        let error = UserSafeError::new(
            ErrorCategory::Platform,
            "device AA:BB:CC:DD:EE:FF failed at C:\\private\\capture",
        );
        assert_eq!(error.message(), "device <redacted> failed at <redacted>");
    }
}
