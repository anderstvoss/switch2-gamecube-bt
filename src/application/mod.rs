//! Backend-independent application orchestration.

use std::time::Duration;

use crate::{
    backend::{AdapterInfo, ControllerInfo, PlatformBackend, ReportObservation},
    domain::{CancellationToken, ConnectionState, ControllerId, Deadline, UserSafeError},
    protocol::InputFrame,
};

/// Default bounded hardware operation timeout.
pub const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(10);

/// Application service over one selected platform backend.
pub struct ControllerService<B> {
    backend: B,
    timeout: Duration,
    cancellation: CancellationToken,
}

impl<B: PlatformBackend> ControllerService<B> {
    /// Creates a service with the default bounded timeout.
    #[must_use]
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            timeout: DEFAULT_OPERATION_TIMEOUT,
            cancellation: CancellationToken::default(),
        }
    }

    /// Overrides the operation timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Returns a token that can cancel current and future waiting operations.
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    /// Lists adapters.
    ///
    /// # Errors
    ///
    /// Returns a backend error.
    pub fn adapters(&mut self) -> Result<Vec<AdapterInfo>, UserSafeError> {
        self.backend.adapters()
    }

    /// Scans for controllers.
    ///
    /// # Errors
    ///
    /// Returns a backend error or timeout.
    pub fn scan(&mut self) -> Result<Vec<ControllerInfo>, UserSafeError> {
        self.backend.scan(self.deadline(), &self.cancellation)
    }

    /// Pairs a controller.
    ///
    /// # Errors
    ///
    /// Returns a backend pairing error or timeout.
    pub fn pair(&mut self, id: &ControllerId) -> Result<ConnectionState, UserSafeError> {
        self.backend.pair(id, self.deadline(), &self.cancellation)
    }

    /// Connects a controller and verifies HID readiness.
    ///
    /// # Errors
    ///
    /// Returns a backend connection error or timeout.
    pub fn connect(&mut self, id: &ControllerId) -> Result<ConnectionState, UserSafeError> {
        self.backend
            .connect(id, self.deadline(), &self.cancellation)
    }

    /// Disconnects a controller.
    ///
    /// # Errors
    ///
    /// Returns a backend error.
    pub fn disconnect(&mut self, id: &ControllerId) -> Result<ConnectionState, UserSafeError> {
        self.backend.disconnect(id)
    }

    /// Returns sanitized controller status.
    ///
    /// # Errors
    ///
    /// Returns a backend lookup error.
    pub fn info(&mut self, id: &ControllerId) -> Result<ControllerInfo, UserSafeError> {
        self.backend.info(id)
    }

    /// Observes bounded report metadata.
    ///
    /// # Errors
    ///
    /// Returns a backend input error or timeout.
    pub fn observe(
        &mut self,
        id: &ControllerId,
        limit: usize,
    ) -> Result<Vec<ReportObservation>, UserSafeError> {
        self.backend
            .observe(id, limit.min(256), self.deadline(), &self.cancellation)
    }

    /// Reads bounded normalized input frames.
    ///
    /// # Errors
    ///
    /// Returns a backend input error or timeout.
    pub fn input(
        &mut self,
        id: &ControllerId,
        limit: usize,
    ) -> Result<Vec<InputFrame>, UserSafeError> {
        self.backend
            .input(id, limit.min(256), self.deadline(), &self.cancellation)
    }

    fn deadline(&self) -> Deadline {
        Deadline::after(self.timeout)
    }
}
