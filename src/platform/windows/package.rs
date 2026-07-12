//! Windows package-identity diagnostics for capability-host verification.

use windows::ApplicationModel::Package;

/// Whether the current process has Windows package identity.
///
/// This deliberately exposes no package name, install path, or other local
/// machine data. It is used only to prove that a capability-host diagnostic was
/// launched through registered package activation.
#[must_use]
pub fn has_package_identity() -> bool {
    Package::Current().is_ok()
}
