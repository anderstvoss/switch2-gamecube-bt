//! Deterministic backend for application and CLI contract tests.

use std::collections::BTreeMap;

use crate::{
    domain::{
        AdapterId, CancellationToken, Capability, CapabilitySet, ConnectionState, ControllerId,
        Deadline, ErrorCategory, UserSafeError,
    },
    protocol::{Axis, Button, InputFrame, OutputRequest},
};

use super::{AdapterInfo, ControllerInfo, PlatformBackend, ReportObservation};

/// Failure injected into the next fake operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeFailure {
    /// Simulated deadline expiry.
    Timeout,
    /// Simulated permission denial.
    PermissionDenied,
    /// Simulated pairing rejection.
    Pairing,
    /// Simulated connection failure.
    Connection,
    /// Simulated HID readiness failure.
    HidUnavailable,
}

/// A deterministic in-memory platform backend.
pub struct FakeBackend {
    adapter: AdapterInfo,
    controllers: BTreeMap<ControllerId, ControllerInfo>,
    next_failure: Option<FakeFailure>,
}

impl Default for FakeBackend {
    fn default() -> Self {
        let capabilities = CapabilitySet::from_capabilities([
            Capability::AdapterInventory,
            Capability::Discovery,
            Capability::Pairing,
            Capability::Connection,
            Capability::HidInventory,
            Capability::InputReports,
        ]);
        let adapter = AdapterInfo {
            id: AdapterId::new("fake-adapter").expect("static fake identifier"),
            label: "Deterministic fake adapter".into(),
            capabilities,
        };
        let controller = ControllerInfo {
            id: ControllerId::new("fake-bee-021").expect("static fake identifier"),
            label: "BEE-021 simulated controller".into(),
            state: ConnectionState::Discovered,
        };
        Self {
            adapter,
            controllers: [(controller.id.clone(), controller)].into(),
            next_failure: None,
        }
    }
}

impl FakeBackend {
    /// Injects a failure into the next fallible operation.
    pub const fn fail_next(&mut self, failure: FakeFailure) {
        self.next_failure = Some(failure);
    }

    fn check_failure(&mut self) -> Result<(), UserSafeError> {
        let Some(failure) = self.next_failure.take() else {
            return Ok(());
        };
        let (category, message) = match failure {
            FakeFailure::Timeout => (ErrorCategory::Timeout, "simulated timeout"),
            FakeFailure::PermissionDenied => (
                ErrorCategory::PermissionDenied,
                "simulated permission denial",
            ),
            FakeFailure::Pairing => (ErrorCategory::PairingFailed, "simulated pairing failure"),
            FakeFailure::Connection => (
                ErrorCategory::ConnectionFailed,
                "simulated connection failure",
            ),
            FakeFailure::HidUnavailable => (ErrorCategory::HidUnavailable, "simulated HID failure"),
        };
        Err(UserSafeError::new(category, message))
    }

    fn controller_mut(
        &mut self,
        controller: &ControllerId,
    ) -> Result<&mut ControllerInfo, UserSafeError> {
        self.controllers.get_mut(controller).ok_or_else(|| {
            UserSafeError::new(ErrorCategory::InvalidData, "unknown controller identifier")
        })
    }

    fn check_deadline(deadline: Deadline) -> Result<(), UserSafeError> {
        if deadline.has_elapsed() {
            Err(UserSafeError::new(
                ErrorCategory::Timeout,
                "operation timed out",
            ))
        } else {
            Ok(())
        }
    }

    fn check_cancellation(cancellation: &CancellationToken) -> Result<(), UserSafeError> {
        if cancellation.is_cancelled() {
            Err(UserSafeError::new(
                ErrorCategory::Cancelled,
                "operation cancelled",
            ))
        } else {
            Ok(())
        }
    }
}

impl PlatformBackend for FakeBackend {
    fn capabilities(&self) -> CapabilitySet {
        self.adapter.capabilities.clone()
    }

    fn adapters(&mut self) -> Result<Vec<AdapterInfo>, UserSafeError> {
        self.check_failure()?;
        Ok(vec![self.adapter.clone()])
    }

    fn scan(
        &mut self,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<Vec<ControllerInfo>, UserSafeError> {
        self.check_failure()?;
        Self::check_cancellation(cancellation)?;
        Self::check_deadline(deadline)?;
        Ok(self.controllers.values().cloned().collect())
    }

    fn pair(
        &mut self,
        controller: &ControllerId,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<ConnectionState, UserSafeError> {
        self.check_failure()?;
        Self::check_cancellation(cancellation)?;
        Self::check_deadline(deadline)?;
        let controller = self.controller_mut(controller)?;
        controller.state = controller
            .state
            .transition(ConnectionState::Pairing)
            .map_err(|error| UserSafeError::new(ErrorCategory::InvalidData, error.to_string()))?;
        controller.state = controller
            .state
            .transition(ConnectionState::Paired)
            .map_err(|error| UserSafeError::new(ErrorCategory::InvalidData, error.to_string()))?;
        Ok(controller.state)
    }

    fn connect(
        &mut self,
        controller: &ControllerId,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<ConnectionState, UserSafeError> {
        self.check_failure()?;
        Self::check_cancellation(cancellation)?;
        Self::check_deadline(deadline)?;
        let controller = self.controller_mut(controller)?;
        if controller.state == ConnectionState::Discovered {
            controller.state = ConnectionState::Paired;
        }
        controller.state = controller
            .state
            .transition(ConnectionState::Connecting)
            .and_then(|state| state.transition(ConnectionState::Connected))
            .and_then(|state| state.transition(ConnectionState::HidReady))
            .map_err(|error| UserSafeError::new(ErrorCategory::InvalidData, error.to_string()))?;
        Ok(controller.state)
    }

    fn disconnect(&mut self, controller: &ControllerId) -> Result<ConnectionState, UserSafeError> {
        self.check_failure()?;
        let controller = self.controller_mut(controller)?;
        controller.state = controller
            .state
            .transition(ConnectionState::Disconnected)
            .map_err(|error| UserSafeError::new(ErrorCategory::InvalidData, error.to_string()))?;
        Ok(controller.state)
    }

    fn info(&mut self, controller: &ControllerId) -> Result<ControllerInfo, UserSafeError> {
        self.check_failure()?;
        self.controllers.get(controller).cloned().ok_or_else(|| {
            UserSafeError::new(ErrorCategory::InvalidData, "unknown controller identifier")
        })
    }

    fn observe(
        &mut self,
        controller: &ControllerId,
        limit: usize,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<Vec<ReportObservation>, UserSafeError> {
        self.check_failure()?;
        Self::check_cancellation(cancellation)?;
        Self::check_deadline(deadline)?;
        let _ = self.info(controller)?;
        Ok((0..limit.min(4))
            .map(|index| ReportObservation {
                report_id: 0x30,
                length: 16 + index,
            })
            .collect())
    }

    fn input(
        &mut self,
        controller: &ControllerId,
        limit: usize,
        deadline: Deadline,
        cancellation: &CancellationToken,
    ) -> Result<Vec<InputFrame>, UserSafeError> {
        self.check_failure()?;
        Self::check_cancellation(cancellation)?;
        Self::check_deadline(deadline)?;
        let _ = self.info(controller)?;
        let mut frame = InputFrame::default();
        frame.buttons.insert(Button::A);
        frame.axes.insert(Axis::LeftX, 12_000);
        Ok(vec![frame; limit.min(4)])
    }

    fn output(
        &mut self,
        controller: &ControllerId,
        request: OutputRequest,
    ) -> Result<(), UserSafeError> {
        self.check_failure()?;
        let _ = self.info(controller)?;
        match request {
            OutputRequest::Deny => Ok(()),
            OutputRequest::VerifiedVolatile(_) => Err(UserSafeError::new(
                ErrorCategory::Unsupported,
                "fake backend has no verified outputs",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn deadline() -> Deadline {
        Deadline::after(Duration::from_secs(1))
    }

    fn active() -> CancellationToken {
        CancellationToken::default()
    }

    #[test]
    fn fake_completes_pair_connect_and_disconnect() {
        let mut backend = FakeBackend::default();
        let id = ControllerId::new("fake-bee-021").expect("static id");
        assert_eq!(
            backend.pair(&id, deadline(), &active()),
            Ok(ConnectionState::Paired)
        );
        assert_eq!(
            backend.connect(&id, deadline(), &active()),
            Ok(ConnectionState::HidReady)
        );
        assert_eq!(backend.disconnect(&id), Ok(ConnectionState::Disconnected));
    }

    #[test]
    fn injected_failures_are_one_shot() {
        let mut backend = FakeBackend::default();
        backend.fail_next(FakeFailure::PermissionDenied);
        let error = backend.adapters().expect_err("injected failure");
        assert_eq!(error.category(), ErrorCategory::PermissionDenied);
        assert_eq!(backend.adapters().expect("failure consumed").len(), 1);
    }

    #[test]
    fn outputs_remain_default_deny() {
        let mut backend = FakeBackend::default();
        let id = ControllerId::new("fake-bee-021").expect("static id");
        assert_eq!(backend.output(&id, OutputRequest::Deny), Ok(()));
    }

    #[test]
    fn cancellation_prevents_waiting_operations() {
        let mut backend = FakeBackend::default();
        let cancellation = active();
        cancellation.cancel();
        let error = backend
            .scan(deadline(), &cancellation)
            .expect_err("cancelled operation");
        assert_eq!(error.category(), ErrorCategory::Cancelled);
    }
}
