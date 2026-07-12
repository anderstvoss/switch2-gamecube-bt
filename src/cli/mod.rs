//! Diagnostic CLI arguments, rendering, and stable exit categories.

use std::{fmt::Write as _, time::Duration};

use clap::{Parser, Subcommand};
use serde::Serialize;

use crate::{
    application::ControllerService,
    backend::{ControllerInfo, FakeBackend, ReportObservation},
    domain::{ConnectionState, ControllerId, ErrorCategory, IdentifierError, UserSafeError},
    protocol::InputFrame,
};

const JSON_SCHEMA_VERSION: u16 = 1;
const MAX_LIMIT: usize = 256;
#[cfg(windows)]
const NINTENDO_VENDOR_ID: u16 = 0x057e;
#[cfg(windows)]
const BEE_021_USB_PRODUCT_ID: u16 = 0x2073;
#[cfg(windows)]
const BEE_021_BULK_INTERFACE: u8 = 1;

/// Command-line arguments for `s2bt`.
#[derive(Debug, Parser)]
#[command(name = "s2bt", version, about = "Switch 2 controller diagnostic tool")]
pub struct Args {
    /// Emit versioned machine-readable JSON.
    #[arg(long, global = true)]
    pub json: bool,
    /// Operation timeout in seconds.
    #[arg(long, global = true, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=300))]
    pub timeout: u64,
    /// Diagnostic command.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported initial diagnostic commands.
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// List available adapters.
    Adapters,
    /// Scan for nearby candidate controllers.
    Scan,
    /// Pair a selected controller.
    Pair {
        /// Opaque controller identifier returned by scan.
        controller: String,
    },
    /// Connect and verify HID readiness.
    Connect {
        /// Opaque controller identifier returned by scan.
        controller: String,
    },
    /// Disconnect a controller.
    Disconnect {
        /// Opaque controller identifier returned by scan.
        controller: String,
    },
    /// Show sanitized controller information.
    Info {
        /// Opaque controller identifier returned by scan.
        controller: String,
    },
    /// Observe bounded report identifiers and lengths.
    Observe {
        /// Opaque controller identifier returned by scan.
        controller: String,
        /// Maximum number of observations.
        #[arg(long, default_value_t = 4, value_parser = parse_limit)]
        limit: usize,
    },
    /// Read bounded normalized input frames.
    InputTest {
        /// Opaque controller identifier returned by scan.
        controller: String,
        /// Maximum number of input frames.
        #[arg(long, default_value_t = 4, value_parser = parse_limit)]
        limit: usize,
    },
    /// Produce a bounded sanitized diagnostic summary.
    Diagnose {
        /// Opaque controller identifier returned by scan.
        controller: String,
    },
    /// Enumerate BEE-021 USB HID metadata without opening the device.
    #[cfg(windows)]
    UsbInventory,
    /// Inspect BEE-021 `WinUSB` bulk endpoints without claiming the interface.
    #[cfg(windows)]
    UsbBulkInventory,
    /// Run the reviewed one-packet BEE-021 input probe.
    #[cfg(windows)]
    UsbInputProbe {
        /// Confirm the reviewed host-to-controller start-stream write.
        #[arg(long)]
        approve_reviewed_write: bool,
        /// Bounded input observation duration in seconds.
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=10))]
        seconds: u64,
        /// Maximum number of report metadata entries to retain.
        #[arg(long, default_value_t = 64, value_parser = parse_limit)]
        limit: usize,
    },
    /// Fingerprint the BEE-021 HID descriptor without exposing its bytes.
    #[cfg(windows)]
    UsbDescriptor,
    /// Observe bounded BEE-021 input report metadata without exposing bytes.
    #[cfg(windows)]
    UsbObserve {
        /// Observation duration in seconds.
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=60))]
        seconds: u64,
        /// Maximum number of reports to aggregate.
        #[arg(long, default_value_t = 256, value_parser = parse_limit)]
        limit: usize,
    },
}

/// Runs the CLI against the deterministic backend.
#[must_use]
pub fn run(args: Args) -> CliResult {
    let backend_name = match args.command {
        #[cfg(windows)]
        Command::UsbInventory
        | Command::UsbBulkInventory
        | Command::UsbDescriptor
        | Command::UsbObserve { .. } => "windows_usb_read_only",
        #[cfg(windows)]
        Command::UsbInputProbe { .. } => "windows_usb_reviewed_experiment",
        _ => "fake",
    };
    let mut service = ControllerService::new(FakeBackend::default())
        .with_timeout(Duration::from_secs(args.timeout));
    match execute(&mut service, args.command) {
        Ok(payload) => CliResult {
            exit_code: 0,
            output: render_success(args.json, backend_name, payload),
        },
        Err(error) => CliResult {
            exit_code: exit_code(error.category()),
            output: render_error(args.json, backend_name, &error),
        },
    }
}

/// CLI process result independent of process termination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliResult {
    /// Stable process exit code.
    pub exit_code: u8,
    /// Complete human or JSON output.
    pub output: String,
}

fn parse_limit(value: &str) -> Result<usize, String> {
    let limit = value
        .parse::<usize>()
        .map_err(|_| "limit must be an integer".to_owned())?;
    if (1..=MAX_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(format!("limit must be between 1 and {MAX_LIMIT}"))
    }
}

#[derive(Debug, Serialize)]
struct JsonEnvelope<T> {
    schema_version: u16,
    backend: &'static str,
    status: &'static str,
    data: T,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Payload {
    Adapters {
        items: Vec<AdapterView>,
    },
    Controllers {
        items: Vec<ControllerView>,
    },
    State {
        state: String,
    },
    Controller {
        controller: ControllerView,
    },
    Observations {
        items: Vec<ObservationView>,
    },
    Input {
        frames: Vec<InputView>,
    },
    Diagnostic {
        controller: ControllerView,
        privacy: &'static str,
    },
    #[cfg(windows)]
    UsbInterfaces {
        items: Vec<UsbInterfaceView>,
    },
    #[cfg(windows)]
    UsbBulkInterface {
        interface_number: u8,
        input_endpoint: String,
        output_endpoint: String,
        input_max_packet_size: usize,
        output_max_packet_size: usize,
    },
    #[cfg(windows)]
    UsbDescriptor {
        length: usize,
        sha256: String,
    },
    #[cfg(windows)]
    UsbInputMetadata {
        items: Vec<UsbInputMetadataView>,
    },
    #[cfg(windows)]
    UsbInputProbe {
        command_reply_length: usize,
        reports: Vec<UsbInputMetadataView>,
    },
}

#[derive(Debug, Serialize)]
struct AdapterView {
    label: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ControllerView {
    id: String,
    label: String,
    state: String,
}

#[derive(Debug, Serialize)]
struct ObservationView {
    report_id: u8,
    length: usize,
}

#[derive(Debug, Serialize)]
struct InputView {
    buttons: Vec<String>,
    axes: Vec<(String, i16)>,
    motion_samples: usize,
    battery_percent: Option<u8>,
}

#[cfg(windows)]
#[derive(Debug, Serialize)]
struct UsbInterfaceView {
    vendor_id: String,
    product_id: String,
    usage_page: String,
    usage: String,
    interface_number: i32,
    product_label: Option<String>,
    manufacturer_label: Option<String>,
    bus_type: &'static str,
}

#[cfg(windows)]
#[derive(Debug, Serialize)]
struct UsbInputMetadataView {
    report_id: String,
    length: usize,
    count: usize,
}

fn execute(
    service: &mut ControllerService<FakeBackend>,
    command: Command,
) -> Result<Payload, UserSafeError> {
    match command {
        Command::Adapters => Ok(Payload::Adapters {
            items: service
                .adapters()?
                .into_iter()
                .map(|adapter| AdapterView {
                    label: adapter.label,
                    capabilities: adapter
                        .capabilities
                        .iter()
                        .map(|value| format!("{value:?}"))
                        .collect(),
                })
                .collect(),
        }),
        Command::Scan => Ok(Payload::Controllers {
            items: service.scan()?.into_iter().map(controller_view).collect(),
        }),
        Command::Pair { controller } => Ok(Payload::State {
            state: state_name(service.pair(&parse_id(controller)?)?).into(),
        }),
        Command::Connect { controller } => Ok(Payload::State {
            state: state_name(service.connect(&parse_id(controller)?)?).into(),
        }),
        Command::Disconnect { controller } => Ok(Payload::State {
            state: state_name(service.disconnect(&parse_id(controller)?)?).into(),
        }),
        Command::Info { controller } => Ok(Payload::Controller {
            controller: controller_view(service.info(&parse_id(controller)?)?),
        }),
        Command::Observe { controller, limit } => Ok(Payload::Observations {
            items: service
                .observe(&parse_id(controller)?, limit)?
                .into_iter()
                .map(observation_view)
                .collect(),
        }),
        Command::InputTest { controller, limit } => Ok(Payload::Input {
            frames: service
                .input(&parse_id(controller)?, limit)?
                .into_iter()
                .map(input_view)
                .collect(),
        }),
        Command::Diagnose { controller } => Ok(Payload::Diagnostic {
            controller: controller_view(service.info(&parse_id(controller)?)?),
            privacy: "sanitized",
        }),
        #[cfg(windows)]
        Command::UsbInventory => Ok(Payload::UsbInterfaces {
            items: crate::platform::windows::enumerate_usb_hid(
                NINTENDO_VENDOR_ID,
                Some(BEE_021_USB_PRODUCT_ID),
            )?
            .into_iter()
            .map(|interface| UsbInterfaceView {
                vendor_id: format!("{:04x}", interface.vendor_id),
                product_id: format!("{:04x}", interface.product_id),
                usage_page: format!("{:04x}", interface.usage_page),
                usage: format!("{:04x}", interface.usage),
                interface_number: interface.interface_number,
                product_label: interface.product_label,
                manufacturer_label: interface.manufacturer_label,
                bus_type: interface.bus_type,
            })
            .collect(),
        }),
        #[cfg(windows)]
        Command::UsbBulkInventory => usb_bulk_inventory(),
        #[cfg(windows)]
        Command::UsbInputProbe {
            approve_reviewed_write,
            seconds,
            limit,
        } => usb_input_probe(approve_reviewed_write, seconds, limit),
        #[cfg(windows)]
        Command::UsbDescriptor => {
            let descriptor = crate::platform::windows::inspect_usb_descriptor(
                NINTENDO_VENDOR_ID,
                BEE_021_USB_PRODUCT_ID,
            )?;
            Ok(Payload::UsbDescriptor {
                length: descriptor.length,
                sha256: descriptor.sha256,
            })
        }
        #[cfg(windows)]
        Command::UsbObserve { seconds, limit } => usb_observe(seconds, limit),
    }
}

#[cfg(windows)]
fn usb_bulk_inventory() -> Result<Payload, UserSafeError> {
    let layout = crate::platform::windows::inspect_bulk_endpoints(
        NINTENDO_VENDOR_ID,
        BEE_021_USB_PRODUCT_ID,
        BEE_021_BULK_INTERFACE,
    )?;
    Ok(Payload::UsbBulkInterface {
        interface_number: layout.interface_number,
        input_endpoint: format!("{:02x}", layout.input_endpoint),
        output_endpoint: format!("{:02x}", layout.output_endpoint),
        input_max_packet_size: layout.input_max_packet_size,
        output_max_packet_size: layout.output_max_packet_size,
    })
}

#[cfg(windows)]
fn usb_input_probe(approved: bool, seconds: u64, limit: usize) -> Result<Payload, UserSafeError> {
    if !approved {
        return Err(UserSafeError::new(
            ErrorCategory::PermissionDenied,
            "the reviewed USB write requires --approve-reviewed-write",
        ));
    }
    let observation = crate::platform::windows::run_minimal_input_probe(
        NINTENDO_VENDOR_ID,
        BEE_021_USB_PRODUCT_ID,
        BEE_021_BULK_INTERFACE,
        Duration::from_secs(seconds),
        limit,
    )?;
    Ok(Payload::UsbInputProbe {
        command_reply_length: observation.command_reply_length,
        reports: observation
            .reports
            .into_iter()
            .map(|report| UsbInputMetadataView {
                report_id: format!("{:02x}", report.report_id),
                length: report.length,
                count: 1,
            })
            .collect(),
    })
}

#[cfg(windows)]
fn usb_observe(seconds: u64, limit: usize) -> Result<Payload, UserSafeError> {
    Ok(Payload::UsbInputMetadata {
        items: crate::platform::windows::observe_usb_input(
            NINTENDO_VENDOR_ID,
            BEE_021_USB_PRODUCT_ID,
            Duration::from_secs(seconds),
            limit,
        )?
        .into_iter()
        .map(|observation| UsbInputMetadataView {
            report_id: format!("{:02x}", observation.report_id),
            length: observation.length,
            count: observation.count,
        })
        .collect(),
    })
}

fn parse_id(value: String) -> Result<ControllerId, UserSafeError> {
    ControllerId::new(value).map_err(|error| {
        let message = match error {
            IdentifierError::Empty => "controller identifier is empty",
            IdentifierError::TooLong => "controller identifier is too long",
            IdentifierError::ControlCharacter => "controller identifier contains control data",
        };
        UserSafeError::new(ErrorCategory::InvalidData, message)
    })
}

fn controller_view(controller: ControllerInfo) -> ControllerView {
    ControllerView {
        id: controller.id.as_str().into(),
        label: controller.label,
        state: state_name(controller.state).into(),
    }
}

const fn state_name(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Unknown => "unknown",
        ConnectionState::Discovered => "discovered",
        ConnectionState::Pairing => "pairing",
        ConnectionState::Paired => "paired",
        ConnectionState::Connecting => "connecting",
        ConnectionState::Connected => "connected",
        ConnectionState::HidReady => "hid_ready",
        ConnectionState::Disconnected => "disconnected",
        ConnectionState::Error => "error",
    }
}

const fn observation_view(observation: ReportObservation) -> ObservationView {
    ObservationView {
        report_id: observation.report_id,
        length: observation.length,
    }
}

fn input_view(frame: InputFrame) -> InputView {
    InputView {
        buttons: frame
            .buttons
            .into_iter()
            .map(|button| format!("{button:?}"))
            .collect(),
        axes: frame
            .axes
            .into_iter()
            .map(|(axis, value)| (format!("{axis:?}"), value))
            .collect(),
        motion_samples: frame.motion.len(),
        battery_percent: frame.battery.map(crate::protocol::BatteryLevel::percent),
    }
}

fn render_success(json: bool, backend: &'static str, payload: Payload) -> String {
    if json {
        return serde_json::to_string_pretty(&JsonEnvelope {
            schema_version: JSON_SCHEMA_VERSION,
            backend,
            status: "ok",
            data: payload,
        })
        .expect("serializing known CLI types cannot fail");
    }
    render_human(payload)
}

fn render_human(payload: Payload) -> String {
    let mut output = String::new();
    match payload {
        Payload::Adapters { items } => {
            for adapter in items {
                let _ = writeln!(
                    output,
                    "{}: {}",
                    adapter.label,
                    adapter.capabilities.join(", ")
                );
            }
        }
        Payload::Controllers { items } => {
            for controller in items {
                let _ = writeln!(
                    output,
                    "{} ({}) [{}]",
                    controller.label, controller.id, controller.state
                );
            }
        }
        Payload::State { state } => {
            let _ = writeln!(output, "state: {state}");
        }
        Payload::Controller { controller } | Payload::Diagnostic { controller, .. } => {
            let _ = writeln!(
                output,
                "{} ({}) [{}]",
                controller.label, controller.id, controller.state
            );
        }
        Payload::Observations { items } => {
            for item in items {
                let _ = writeln!(
                    output,
                    "report 0x{:02x}: {} bytes",
                    item.report_id, item.length
                );
            }
        }
        Payload::Input { frames } => {
            for (index, frame) in frames.into_iter().enumerate() {
                let _ = writeln!(
                    output,
                    "frame {index}: buttons={:?} axes={:?}",
                    frame.buttons, frame.axes
                );
            }
        }
        #[cfg(windows)]
        Payload::UsbInterfaces { items } => {
            for item in items {
                let label = item
                    .product_label
                    .as_deref()
                    .unwrap_or("unlabeled HID interface");
                let _ = writeln!(
                    output,
                    "{label}: {:04}:{:04} usage={}:{} interface={} bus={}",
                    item.vendor_id,
                    item.product_id,
                    item.usage_page,
                    item.usage,
                    item.interface_number,
                    item.bus_type
                );
            }
        }
        #[cfg(windows)]
        Payload::UsbBulkInterface {
            interface_number,
            input_endpoint,
            output_endpoint,
            input_max_packet_size,
            output_max_packet_size,
        } => {
            let _ = writeln!(
                output,
                "interface {interface_number}: bulk-in=0x{input_endpoint} ({input_max_packet_size} bytes) bulk-out=0x{output_endpoint} ({output_max_packet_size} bytes)"
            );
        }
        #[cfg(windows)]
        Payload::UsbDescriptor { length, sha256 } => {
            render_usb_descriptor(&mut output, length, &sha256);
        }
        #[cfg(windows)]
        Payload::UsbInputMetadata { items } => render_usb_input_metadata(&mut output, items),
        #[cfg(windows)]
        Payload::UsbInputProbe {
            command_reply_length,
            reports,
        } => render_usb_input_probe(&mut output, command_reply_length, reports),
    }
    output
}

#[cfg(windows)]
fn render_usb_descriptor(output: &mut String, length: usize, sha256: &str) {
    let _ = writeln!(output, "descriptor: {length} bytes sha256={sha256}");
}

#[cfg(windows)]
fn render_usb_input_metadata(output: &mut String, items: Vec<UsbInputMetadataView>) {
    for item in items {
        let _ = writeln!(
            output,
            "report 0x{}: {} bytes, {} observed",
            item.report_id, item.length, item.count
        );
    }
}

#[cfg(windows)]
fn render_usb_input_probe(
    output: &mut String,
    command_reply_length: usize,
    reports: Vec<UsbInputMetadataView>,
) {
    let _ = writeln!(output, "command reply: {command_reply_length} bytes");
    for report in reports {
        let _ = writeln!(
            output,
            "report 0x{}: {} bytes",
            report.report_id, report.length
        );
    }
}

fn render_error(json: bool, backend: &'static str, error: &UserSafeError) -> String {
    if json {
        return serde_json::to_string_pretty(&JsonEnvelope {
            schema_version: JSON_SCHEMA_VERSION,
            backend,
            status: "error",
            data: serde_json::json!({
                "category": format!("{:?}", error.category()),
                "message": error.message(),
            }),
        })
        .expect("serializing known CLI types cannot fail");
    }
    format!("error: {}\n", error.message())
}

const fn exit_code(category: ErrorCategory) -> u8 {
    match category {
        ErrorCategory::Unsupported => 3,
        ErrorCategory::PermissionDenied => 4,
        ErrorCategory::Timeout => 5,
        ErrorCategory::Cancelled => 6,
        ErrorCategory::PairingFailed => 7,
        ErrorCategory::ConnectionFailed => 8,
        ErrorCategory::HidUnavailable => 9,
        ErrorCategory::InvalidData => 10,
        ErrorCategory::Platform => 11,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_has_human_and_json_rendering() {
        let human = run(Args {
            json: false,
            timeout: 1,
            command: Command::Scan,
        });
        assert_eq!(human.exit_code, 0);
        assert!(human.output.contains("BEE-021 simulated controller"));

        let json = run(Args {
            json: true,
            timeout: 1,
            command: Command::Scan,
        });
        let value: serde_json::Value = serde_json::from_str(&json.output).expect("valid JSON");
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["backend"], "fake");
        assert_eq!(value["status"], "ok");
    }

    #[test]
    fn input_is_bounded() {
        let result = run(Args {
            json: false,
            timeout: 1,
            command: Command::InputTest {
                controller: "fake-bee-021".into(),
                limit: MAX_LIMIT,
            },
        });
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output.lines().count(), 4);
    }

    #[cfg(windows)]
    #[test]
    fn usb_input_probe_requires_explicit_write_confirmation() {
        let result = run(Args {
            json: false,
            timeout: 1,
            command: Command::UsbInputProbe {
                approve_reviewed_write: false,
                seconds: 1,
                limit: 1,
            },
        });
        assert_eq!(result.exit_code, 4);
        assert!(result.output.contains("--approve-reviewed-write"));
    }
}
