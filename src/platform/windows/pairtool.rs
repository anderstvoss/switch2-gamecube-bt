//! Sanitized availability check for the Windows `PairTool` lab diagnostic.

use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};

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

/// Sanitized result of a bounded active Bluetooth Classic discovery scan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairToolDiscoveryObservation {
    /// Requested active scan duration.
    pub duration: Duration,
    /// Short digests of discovered Bluetooth Classic endpoint identifiers.
    pub endpoint_digests: Vec<String>,
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

/// Run `PairTool`'s active Bluetooth Classic discovery without pairing.
///
/// # Errors
///
/// Returns a sanitized error if the requested duration is outside one to ten
/// seconds, the diagnostic cannot run, or its bounded process execution fails.
pub fn discover_with_pairtool(
    duration: Duration,
) -> Result<PairToolDiscoveryObservation, UserSafeError> {
    if !(Duration::from_secs(1)..=Duration::from_secs(10)).contains(&duration) {
        return Err(UserSafeError::new(
            ErrorCategory::InvalidData,
            "PairTool discovery duration must be between one and ten seconds",
        ));
    }
    let mut child = Command::new(pairtool_path()?)
        .args(["/enum-endpoints", "/protocol", "Bluetooth", "/continuous"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(platform_error)?;
    let deadline = Instant::now() + duration;
    loop {
        if child.try_wait().map_err(platform_error)?.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().map_err(platform_error)?;
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    let output = child.wait_with_output().map_err(platform_error)?;
    Ok(PairToolDiscoveryObservation {
        duration,
        endpoint_digests: parse_bluetooth_endpoint_digests(&String::from_utf8_lossy(
            &output.stdout,
        )),
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

fn parse_bluetooth_endpoint_digests(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("Bluetooth#"))
        .map(digest)
        .collect()
}

fn digest(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let mut result = String::with_capacity(16);
    for byte in &digest[..8] {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn platform_error(error: impl std::fmt::Display) -> UserSafeError {
    UserSafeError::new(
        ErrorCategory::Platform,
        format!("Windows PairTool inspection failed: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::{has_classic_bluetooth_protocol, parse_bluetooth_endpoint_digests};

    #[test]
    fn recognizes_the_classic_bluetooth_protocol_without_retaining_tool_output() {
        let output = "Protocol ID:                Bluetooth\nUniversal Protocol ID:      {e0cbf06c-cd8b-4647-bb8a-263b43f0f974}";
        assert!(has_classic_bluetooth_protocol(output));
        assert!(!has_classic_bluetooth_protocol("Protocol ID: BluetoothLE"));
    }

    #[test]
    fn endpoint_parser_retains_only_short_digests() {
        let endpoints = parse_bluetooth_endpoint_digests(
            "Bluetooth#sensitive-endpoint\n  ignored property\nBluetooth#another-endpoint",
        );
        assert_eq!(endpoints.len(), 2);
        assert!(endpoints.iter().all(|digest| digest.len() == 16));
        assert!(endpoints.iter().all(|digest| !digest.contains("Bluetooth")));
    }
}
