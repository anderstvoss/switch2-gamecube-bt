//! Core library placeholder for switch2-gamecube-bt.
//!
//! Keep production code small, tested, and free of unsafe Rust unless the
//! project explicitly revisits that policy.

/// Returns the project identifier used in logs and diagnostics.
#[must_use]
pub const fn project_name() -> &'static str {
    "switch2-gamecube-bt"
}

#[cfg(test)]
mod tests {
    use super::project_name;

    #[test]
    fn project_name_is_stable() {
        assert_eq!(project_name(), "switch2-gamecube-bt");
    }
}
