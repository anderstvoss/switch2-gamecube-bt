//! Safety-gated model of the BEE-021 USB initialization protocol.
//!
//! This module does not provide a live USB transport. Its production plan is
//! deliberately non-executable while audited upstream steps remain
//! unclassified.

use std::fmt;

const MAX_PACKET_LENGTH: usize = 64;
const MAX_REPLY_LENGTH: usize = 64;

/// Audited source revision for the modeled command ordering.
pub const AUDITED_SDL_REVISION: &str = "82141a2439acc111661102ba6f968c85e71cff40";

/// SDL revision that replaced the original one-packet startup with a sequence
/// captured from real hardware.
pub const SDL_FULL_SEQUENCE_REVISION: &str = "70bfdd013a804fdb15ec906d4ba18389c57e9420";

/// SDL revision that removed four apparent console queries from that sequence.
pub const SDL_QUERY_REMOVAL_REVISION: &str = "9fd3dbfc42a247b996858fe66fa835bdb1f03aa3";

/// Returns the exact ten-packet initialization sequence in the pinned SDL
/// revision for an isolated upstream-reference experiment.
///
/// This includes SDL-labeled unknown, rumble, and grip packets. It is kept
/// separate from [`InitializationPlan`] and must never be treated as the normal
/// project allowlist.
pub(crate) fn sdl_reference_packets() -> &'static [&'static [u8]] {
    &[
        &[0x07, 0x91, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
        &[
            0x0c, 0x91, 0x00, 0x02, 0x00, 0x04, 0x00, 0x00, 0x27, 0x00, 0x00, 0x00,
        ],
        &[0x11, 0x91, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
        &[
            0x0a, 0x91, 0x00, 0x08, 0x00, 0x14, 0x00, 0x00, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0x35, 0x00, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x0c, 0x91, 0x00, 0x04, 0x00, 0x04, 0x00, 0x00, 0x27, 0x00, 0x00, 0x00,
        ],
        &[0x01, 0x91, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x00],
        &[0x01, 0x91, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
        &[
            0x08, 0x91, 0x00, 0x02, 0x00, 0x04, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        ],
        &[
            0x03, 0x91, 0x00, 0x0a, 0x00, 0x04, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
        ],
        &[
            0x03, 0x91, 0x00, 0x0d, 0x00, 0x08, 0x00, 0x00, 0x01, 0x00, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ],
    ]
}

/// Persistence classification applied before any command may be serialized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyClass {
    /// A read that does not change controller state.
    ReadOnly,
    /// A setting expected to last only for the current wired session.
    VolatileSession,
    /// Upstream behavior suggests session scope, but persistence is unverified.
    CandidateVolatile,
    /// Semantics or persistence have not been established.
    Unclassified,
    /// Firmware, flash, calibration, pairing, or other persistent mutation.
    PersistentForbidden,
}

/// A classified BEE-021 USB command with an allowlisted encoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClassifiedCommand {
    /// Select which feature-output channels may be used this session.
    SetFeatureOutputMask,
    /// Enable the previously selected feature-output channels.
    EnableFeatureOutputChannels,
    /// Select input report format `0x05` for the current session.
    SetInputReportFormat,
    /// Start the current session's input stream.
    StartInputStream,
}

impl ClassifiedCommand {
    pub(crate) fn packet(self) -> &'static [u8] {
        match self {
            Self::SetFeatureOutputMask => &[
                0x0c, 0x91, 0x00, 0x02, 0x00, 0x04, 0x00, 0x00, 0x27, 0x00, 0x00, 0x00,
            ],
            Self::EnableFeatureOutputChannels => &[
                0x0c, 0x91, 0x00, 0x04, 0x00, 0x04, 0x00, 0x00, 0x27, 0x00, 0x00, 0x00,
            ],
            Self::SetInputReportFormat => &[
                0x03, 0x91, 0x00, 0x0a, 0x00, 0x04, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
            ],
            Self::StartInputStream => &[
                0x03, 0x91, 0x00, 0x0d, 0x00, 0x08, 0x00, 0x00, 0x01, 0x00, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff,
            ],
        }
    }
}

/// A reason the audited upstream sequence cannot yet be executed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitializationBlocker {
    /// Upstream labels one or more required steps as having unknown purpose.
    UnknownPurposeCommand,
    /// Rumble-related setup is not required for input and remains excluded.
    RumbleSetupUnclassified,
    /// Charging-grip behavior is unrelated to BEE-021 and remains excluded.
    NonBee021GripCommand,
}

/// One ordered item in the reviewed initialization plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitializationStep {
    /// A command paired with its current persistence classification.
    Command {
        /// Allowlisted packet encoding.
        command: ClassifiedCommand,
        /// Current review result.
        safety_class: SafetyClass,
    },
    /// An unresolved step that prevents all execution.
    Blocked(InitializationBlocker),
}

/// Immutable initialization plan with mandatory preflight validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializationPlan {
    steps: Box<[InitializationStep]>,
}

impl InitializationPlan {
    /// Returns the safety-gated plan derived from the audited SDL ordering.
    ///
    /// This plan intentionally contains blockers and therefore cannot execute.
    #[must_use]
    pub fn audited_bee021() -> Self {
        Self {
            steps: Box::new([
                InitializationStep::Blocked(InitializationBlocker::UnknownPurposeCommand),
                InitializationStep::Command {
                    command: ClassifiedCommand::SetFeatureOutputMask,
                    safety_class: SafetyClass::CandidateVolatile,
                },
                InitializationStep::Blocked(InitializationBlocker::UnknownPurposeCommand),
                InitializationStep::Blocked(InitializationBlocker::RumbleSetupUnclassified),
                InitializationStep::Command {
                    command: ClassifiedCommand::EnableFeatureOutputChannels,
                    safety_class: SafetyClass::CandidateVolatile,
                },
                InitializationStep::Blocked(InitializationBlocker::UnknownPurposeCommand),
                InitializationStep::Blocked(InitializationBlocker::RumbleSetupUnclassified),
                InitializationStep::Blocked(InitializationBlocker::NonBee021GripCommand),
                InitializationStep::Command {
                    command: ClassifiedCommand::SetInputReportFormat,
                    safety_class: SafetyClass::CandidateVolatile,
                },
                InitializationStep::Command {
                    command: ClassifiedCommand::StartInputStream,
                    safety_class: SafetyClass::CandidateVolatile,
                },
            ]),
        }
    }

    /// Returns the smallest input probe supported by SDL's public history.
    ///
    /// Early wired support used only the start-stream packet, but the upstream
    /// history does not establish whether the command changes persistent
    /// state. It therefore remains candidate-volatile and fails preflight
    /// before any transport operation.
    #[must_use]
    pub fn candidate_minimal_input_probe() -> Self {
        Self {
            steps: Box::new([InitializationStep::Command {
                command: ClassifiedCommand::StartInputStream,
                safety_class: SafetyClass::CandidateVolatile,
            }]),
        }
    }

    /// Returns the next-smallest input probe supported by SDL history.
    ///
    /// This selects the 64-byte report format before starting the stream. Both
    /// writes remain candidate-volatile and normal plan execution rejects them.
    #[must_use]
    pub fn candidate_report5_input_probe() -> Self {
        Self {
            steps: Box::new([
                InitializationStep::Command {
                    command: ClassifiedCommand::SetInputReportFormat,
                    safety_class: SafetyClass::CandidateVolatile,
                },
                InitializationStep::Command {
                    command: ClassifiedCommand::StartInputStream,
                    safety_class: SafetyClass::CandidateVolatile,
                },
            ]),
        }
    }

    /// Returns the four-command probe containing every described non-rumble
    /// initialization candidate.
    #[must_use]
    pub fn candidate_described_input_probe() -> Self {
        Self {
            steps: Box::new([
                InitializationStep::Command {
                    command: ClassifiedCommand::SetFeatureOutputMask,
                    safety_class: SafetyClass::CandidateVolatile,
                },
                InitializationStep::Command {
                    command: ClassifiedCommand::EnableFeatureOutputChannels,
                    safety_class: SafetyClass::CandidateVolatile,
                },
                InitializationStep::Command {
                    command: ClassifiedCommand::SetInputReportFormat,
                    safety_class: SafetyClass::CandidateVolatile,
                },
                InitializationStep::Command {
                    command: ClassifiedCommand::StartInputStream,
                    safety_class: SafetyClass::CandidateVolatile,
                },
            ]),
        }
    }

    /// Returns the ordered plan items for diagnostics and review.
    #[must_use]
    pub fn steps(&self) -> &[InitializationStep] {
        &self.steps
    }

    /// Returns all unresolved blockers without serializing any packet.
    #[must_use]
    pub fn blockers(&self) -> Vec<InitializationBlocker> {
        self.steps
            .iter()
            .filter_map(|step| match step {
                InitializationStep::Blocked(blocker) => Some(*blocker),
                InitializationStep::Command { .. } => None,
            })
            .collect()
    }

    /// Executes a fully classified plan against an abstract bounded transport.
    ///
    /// Preflight checks the complete plan before the first transfer, so a later
    /// blocker can never cause a partially initialized controller.
    ///
    /// # Errors
    ///
    /// Returns [`InitializationError`] before I/O if any step is blocked or has
    /// a disallowed safety class. Transport and reply validation failures stop
    /// execution immediately.
    pub fn execute<T: BulkTransport>(&self, transport: &mut T) -> Result<(), InitializationError> {
        self.preflight()?;
        for step in &self.steps {
            let InitializationStep::Command { command, .. } = step else {
                unreachable!("preflight rejects blocked steps")
            };
            let packet = command.packet();
            if packet.is_empty() || packet.len() > MAX_PACKET_LENGTH {
                return Err(InitializationError::InvalidPacketLength(packet.len()));
            }
            transport
                .send(packet)
                .map_err(InitializationError::Transport)?;
            let reply_length = transport
                .receive(MAX_REPLY_LENGTH)
                .map_err(InitializationError::Transport)?;
            if reply_length == 0 || reply_length > MAX_REPLY_LENGTH {
                return Err(InitializationError::InvalidReplyLength(reply_length));
            }
        }
        Ok(())
    }

    fn preflight(&self) -> Result<(), InitializationError> {
        for step in &self.steps {
            match step {
                InitializationStep::Blocked(blocker) => {
                    return Err(InitializationError::Blocked(*blocker));
                }
                InitializationStep::Command { safety_class, .. }
                    if *safety_class != SafetyClass::VolatileSession =>
                {
                    return Err(InitializationError::DisallowedSafetyClass(*safety_class));
                }
                InitializationStep::Command { .. } => {}
            }
        }
        Ok(())
    }
}

/// Minimal bounded bulk-transfer seam used by mocks and a future live adapter.
pub trait BulkTransport {
    /// Sends one already validated command packet.
    ///
    /// # Errors
    ///
    /// Returns a bounded transport error without device paths or identifiers.
    fn send(&mut self, packet: &[u8]) -> Result<(), TransportError>;

    /// Receives one reply no larger than `maximum_length` and returns its size.
    ///
    /// # Errors
    ///
    /// Returns a bounded transport error without reply contents.
    fn receive(&mut self, maximum_length: usize) -> Result<usize, TransportError>;
}

/// Redacted failure returned by a bulk transport implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportError {
    /// The transfer exceeded its deadline.
    Timeout,
    /// The interface rejected or failed the transfer.
    TransferFailed,
}

/// Safety or transport failure while applying an initialization plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitializationError {
    /// An unresolved audited step prevented all I/O.
    Blocked(InitializationBlocker),
    /// A command was not classified as volatile session state.
    DisallowedSafetyClass(SafetyClass),
    /// A command packet violated the fixed size bound.
    InvalidPacketLength(usize),
    /// A reply was empty or exceeded the requested bound.
    InvalidReplyLength(usize),
    /// The abstract transport failed.
    Transport(TransportError),
}

impl fmt::Display for InitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "BEE-021 initialization rejected: {self:?}")
    }
}

impl std::error::Error for InitializationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockTransport {
        sent: Vec<Vec<u8>>,
        reply_lengths: Vec<usize>,
    }

    impl BulkTransport for MockTransport {
        fn send(&mut self, packet: &[u8]) -> Result<(), TransportError> {
            self.sent.push(packet.to_vec());
            Ok(())
        }

        fn receive(&mut self, _maximum_length: usize) -> Result<usize, TransportError> {
            Ok(self.reply_lengths.pop().unwrap_or(8))
        }
    }

    fn classified_plan() -> InitializationPlan {
        InitializationPlan {
            steps: Box::new([
                InitializationStep::Command {
                    command: ClassifiedCommand::SetFeatureOutputMask,
                    safety_class: SafetyClass::VolatileSession,
                },
                InitializationStep::Command {
                    command: ClassifiedCommand::EnableFeatureOutputChannels,
                    safety_class: SafetyClass::VolatileSession,
                },
                InitializationStep::Command {
                    command: ClassifiedCommand::SetInputReportFormat,
                    safety_class: SafetyClass::VolatileSession,
                },
                InitializationStep::Command {
                    command: ClassifiedCommand::StartInputStream,
                    safety_class: SafetyClass::VolatileSession,
                },
            ]),
        }
    }

    #[test]
    fn audited_plan_is_blocked_before_any_transfer() {
        let plan = InitializationPlan::audited_bee021();
        let mut transport = MockTransport::default();
        assert!(matches!(
            plan.execute(&mut transport),
            Err(InitializationError::Blocked(_))
        ));
        assert!(transport.sent.is_empty());
        assert_eq!(plan.blockers().len(), 6);
    }

    #[test]
    fn minimal_input_probe_remains_non_executable() {
        let plan = InitializationPlan::candidate_minimal_input_probe();
        assert_eq!(
            plan.steps(),
            [InitializationStep::Command {
                command: ClassifiedCommand::StartInputStream,
                safety_class: SafetyClass::CandidateVolatile,
            }]
        );
        let mut transport = MockTransport::default();
        assert_eq!(
            plan.execute(&mut transport),
            Err(InitializationError::DisallowedSafetyClass(
                SafetyClass::CandidateVolatile
            ))
        );
        assert!(transport.sent.is_empty());
    }

    #[test]
    fn report5_input_probe_preserves_order_and_remains_non_executable() {
        let plan = InitializationPlan::candidate_report5_input_probe();
        assert_eq!(
            plan.steps(),
            [
                InitializationStep::Command {
                    command: ClassifiedCommand::SetInputReportFormat,
                    safety_class: SafetyClass::CandidateVolatile,
                },
                InitializationStep::Command {
                    command: ClassifiedCommand::StartInputStream,
                    safety_class: SafetyClass::CandidateVolatile,
                },
            ]
        );
        let mut transport = MockTransport::default();
        assert!(plan.execute(&mut transport).is_err());
        assert!(transport.sent.is_empty());
    }

    #[test]
    fn described_input_probe_excludes_unknown_rumble_and_grip_steps() {
        let plan = InitializationPlan::candidate_described_input_probe();
        assert_eq!(plan.steps().len(), 4);
        assert!(plan.steps().iter().all(|step| matches!(
            step,
            InitializationStep::Command {
                safety_class: SafetyClass::CandidateVolatile,
                ..
            }
        )));
        assert!(plan.blockers().is_empty());
        let mut transport = MockTransport::default();
        assert!(plan.execute(&mut transport).is_err());
        assert!(transport.sent.is_empty());
    }

    #[test]
    fn sdl_reference_sequence_is_fixed_bounded_and_separate() {
        let packets = sdl_reference_packets();
        assert_eq!(packets.len(), 10);
        assert!(
            packets
                .iter()
                .all(|packet| !packet.is_empty() && packet.len() <= MAX_PACKET_LENGTH)
        );
        assert_eq!(packets[0], [0x07, 0x91, 0x00, 0x01, 0, 0, 0, 0]);
        assert_eq!(packets[9], ClassifiedCommand::StartInputStream.packet());
    }

    #[test]
    fn fully_classified_test_plan_preserves_order() {
        let plan = classified_plan();
        let mut transport = MockTransport::default();
        plan.execute(&mut transport).expect("classified mock plan");
        assert_eq!(transport.sent.len(), 4);
        assert_eq!(transport.sent[0][0], 0x0c);
        assert_eq!(transport.sent[2][8], 0x05);
        assert_eq!(transport.sent[3].len(), 16);
    }

    #[test]
    fn invalid_reply_stops_later_commands() {
        let plan = classified_plan();
        let mut transport = MockTransport {
            sent: Vec::new(),
            reply_lengths: vec![8, 0, 8],
        };
        assert_eq!(
            plan.execute(&mut transport),
            Err(InitializationError::InvalidReplyLength(0))
        );
        assert_eq!(transport.sent.len(), 2);
    }
}
