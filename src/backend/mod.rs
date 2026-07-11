//! Platform backend contract and deterministic test backend.

mod fake;

use crate::{
    domain::{
        AdapterId, CancellationToken, CapabilitySet, ConnectionState, ControllerId, Deadline,
        UserSafeError,
    },
    protocol::{InputFrame, OutputRequest, RawReport},
};

pub use fake::{FakeBackend, FakeFailure};

/// Sanitized adapter information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterInfo {
    /// Opaque identifier used only for subsequent operations.
    pub id: AdapterId,
    /// Non-sensitive display label.
    pub label: String,
    /// Operations supported by this adapter.
    pub capabilities: CapabilitySet,
}

/// Sanitized controller information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerInfo {
    /// Opaque identifier used only for subsequent operations.
    pub id: ControllerId,
    /// Non-sensitive display label.
    pub label: String,
    /// Current independently observed state.
    pub state: ConnectionState,
}

/// One bounded report observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReportObservation {
    /// Report identifier.
    pub report_id: u8,
    /// Total report length without report contents.
    pub length: usize,
}

impl From<&RawReport> for ReportObservation {
    fn from(report: &RawReport) -> Self {
        Self {
            report_id: report.report_id(),
            length: report.bytes().len(),
        }
    }
}

/// Synchronous platform boundary used by the initial application service.
pub trait PlatformBackend {
    /// Reports backend-wide capabilities.
    fn capabilities(&self) -> CapabilitySet;

    /// Enumerates sanitized adapter records.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe platform error.
    fn adapters(&mut self) -> Result<Vec<AdapterInfo>, UserSafeError>;

    /// Performs bounded discovery.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe discovery error.
    fn scan(
        &mut self,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<Vec<ControllerInfo>, UserSafeError>;

    /// Requests operating-system pairing.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe pairing error.
    fn pair(
        &mut self,
        controller: &ControllerId,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<ConnectionState, UserSafeError>;

    /// Connects a known controller.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe connection error.
    fn connect(
        &mut self,
        controller: &ControllerId,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<ConnectionState, UserSafeError>;

    /// Disconnects a controller.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe platform error.
    fn disconnect(&mut self, controller: &ControllerId) -> Result<ConnectionState, UserSafeError>;

    /// Returns sanitized controller status.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe lookup error.
    fn info(&mut self, controller: &ControllerId) -> Result<ControllerInfo, UserSafeError>;

    /// Observes bounded report metadata without exposing report contents.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe input error.
    fn observe(
        &mut self,
        controller: &ControllerId,
        limit: usize,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<Vec<ReportObservation>, UserSafeError>;

    /// Reads bounded normalized input.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe input error.
    fn input(
        &mut self,
        controller: &ControllerId,
        limit: usize,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<Vec<InputFrame>, UserSafeError>;

    /// Sends an explicitly policy-gated output request.
    ///
    /// # Errors
    ///
    /// Returns a privacy-safe output error.
    fn output(
        &mut self,
        controller: &ControllerId,
        request: OutputRequest,
    ) -> Result<(), UserSafeError>;
}
