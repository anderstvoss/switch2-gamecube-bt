//! Sanitized availability check for the Windows `PairTool` lab diagnostic.

use std::{env, path::PathBuf, process::Command};

use crate::domain::{ErrorCategory, UserSafeError};

const CLASSIC_PROTOCOL_ID: &str = "{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}";

/// Availability of the Windows `PairTool` Bluetooth Classic diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairToolStatus {
    /// Whether the inbox diagnostic could be executed.
    pub available: bool,
    /// Whether its protocol list included Bluetooth Classic discovery.
    pub classic_bluetooth_available: bool,
}

/// Inspect the installed Windows `PairTool` without starting device discovery.
///
/// `PairTool` is used only to validate the Windows lab. The controller runtime
/// does not depend on it.
///
/// # Errors
///
/// Returns a sanitized platform error when the system directory cannot be
/// resolved or `PairTool` cannot return its protocol inventory.
pub fn inspect_pairtool() -> Result<PairToolStatus, UserSafeError> {
    let executable = pairtool_path()?;
    let output = Command::new(executable)
        .arg("/enum-protocols")
        .output()
        .map_err(platform_error)?;
    if !output.status.success() {
        return Err(UserSafeError::new(
            ErrorCategory::Platform,
            "Windows PairTool protocol inventory failed",
        ));
    }
    let output = String::from_utf8_lossy(&output.stdout);
    Ok(PairToolStatus {
        available: true,
        classic_bluetooth_available: has_classic_bluetooth_protocol(&output),
    })
}

fn pairtool_path() -> Result<PathBuf, UserSafeError> {
    let system_root = env::var_os("SystemRoot").ok_or_else(|| {
        UserSafeError::new(
            ErrorCategory::Unsupported,
            "Windows system directory is unavailable",
        )
    })?;
    Ok(PathBuf::from(system_root)
        .join("System32")
        .join("pairtool.exe"))
}

fn has_classic_bluetooth_protocol(output: &str) -> bool {
    output.contains("Protocol ID:                Bluetooth") && output.contains(CLASSIC_PROTOCOL_ID)
}

fn platform_error(error: impl std::fmt::Display) -> UserSafeError {
    UserSafeError::new(
        ErrorCategory::Platform,
        format!("Windows PairTool inspection failed: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::has_classic_bluetooth_protocol;

    #[test]
    fn recognizes_the_classic_bluetooth_protocol_without_retaining_tool_output() {
        let output = "Protocol ID:                Bluetooth\nUniversal Protocol ID:      {e0cbf06c-cd8b-4647-bb8a-263b43f0f974}";
        assert!(has_classic_bluetooth_protocol(output));
        assert!(!has_classic_bluetooth_protocol("Protocol ID: BluetoothLE"));
    }
}
