//! Portable contracts for Switch 2 controller discovery, transport, decoding,
//! and output.

pub mod application;
pub mod backend;
pub mod cli;
pub mod domain;
pub mod protocol;

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
