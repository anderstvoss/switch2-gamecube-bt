//! Diagnostic CLI arguments, rendering, and stable exit categories.

use std::{fmt::Write as _, path::PathBuf, time::Duration};

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
const MAX_FRAME_LIMIT: usize = 8_192;
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
    /// Write the sanitized command result to a caller-selected local file.
    #[arg(long, global = true)]
    pub result_file: Option<PathBuf>,
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
    /// Enumerate Windows Bluetooth devices without pairing or connecting.
    #[cfg(windows)]
    BluetoothInventory,
    /// Watch for nearby unpaired Bluetooth devices without pairing.
    #[cfg(windows)]
    BluetoothScan {
        /// Bounded scan duration; the BEE-021 pairing window is brief.
        #[arg(long, default_value_t = 8, value_parser = clap::value_parser!(u64).range(1..=10))]
        seconds: u64,
    },
    /// Check the Windows active-discovery lab diagnostic without scanning.
    #[cfg(windows)]
    BluetoothLabStatus,
    /// Run a bounded `PairTool` Bluetooth Classic discovery experiment.
    #[cfg(windows)]
    BluetoothPairtoolScan {
        /// Confirm that this active radio-discovery experiment may run.
        #[arg(long)]
        approve_active_discovery: bool,
        /// Bounded scan duration in seconds.
        #[arg(long, default_value_t = 8, value_parser = clap::value_parser!(u64).range(1..=10))]
        seconds: u64,
    },
    /// Scan BLE advertisements without connecting or pairing.
    #[cfg(windows)]
    BleScan {
        /// Bounded scan duration in seconds.
        #[arg(long, default_value_t = 8, value_parser = clap::value_parser!(u64).range(1..=10))]
        seconds: u64,
    },
    /// Discover unpaired BLE devices without pairing or connecting.
    #[cfg(windows)]
    BleDeviceScan {
        /// Bounded scan duration in seconds.
        #[arg(long, default_value_t = 8, value_parser = clap::value_parser!(u64).range(1..=10))]
        seconds: u64,
    },
    /// Report whether this process has Windows package identity.
    #[cfg(windows)]
    PackageStatus,
    /// Report default adapter BLE capabilities without scanning.
    #[cfg(windows)]
    BleAdapterStatus,
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
    /// Run the reviewed report-format `0x05` plus start-stream probe.
    #[cfg(windows)]
    UsbReport5InputProbe {
        /// Confirm both reviewed host-to-controller writes.
        #[arg(long)]
        approve_reviewed_writes: bool,
        /// Bounded input observation duration in seconds.
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=10))]
        seconds: u64,
        /// Maximum number of report metadata entries to retain.
        #[arg(long, default_value_t = 64, value_parser = parse_limit)]
        limit: usize,
    },
    /// Run all four reviewed, described non-rumble input commands.
    #[cfg(windows)]
    UsbDescribedInputProbe {
        /// Confirm all four reviewed host-to-controller writes.
        #[arg(long)]
        approve_reviewed_writes: bool,
        /// Bounded input observation duration in seconds.
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=10))]
        seconds: u64,
        /// Maximum number of report metadata entries to retain.
        #[arg(long, default_value_t = 64, value_parser = parse_limit)]
        limit: usize,
    },
    /// Run the exact pinned SDL sequence as an isolated reference experiment.
    #[cfg(windows)]
    UsbSdlReferenceProbe {
        /// Confirm the exact ten-packet SDL reference sequence.
        #[arg(long)]
        approve_exact_sdl_sequence: bool,
        /// Bounded input observation duration in seconds.
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=10))]
        seconds: u64,
        /// Maximum number of report metadata entries to retain.
        #[arg(long, default_value_t = 64, value_parser = parse_limit)]
        limit: usize,
    },
    /// Reapply the reviewed motion feature enable after sensor warm-up.
    #[cfg(windows)]
    UsbMotionEnableProbe {
        /// Confirm the reviewed feature-enable write.
        #[arg(long)]
        approve_reviewed_write: bool,
    },
    /// Read documented calibration blocks without exposing their contents.
    #[cfg(windows)]
    UsbCalibrationStatus {
        /// Confirm the documented read-only calibration operation.
        #[arg(long)]
        approve_read_only_calibration: bool,
    },
    /// Exercise decoded input using local read-only calibration.
    #[cfg(windows)]
    UsbCalibratedInputTest {
        /// Confirm the documented read-only calibration operation.
        #[arg(long)]
        approve_read_only_calibration: bool,
        /// Confirm the isolated SDL initialization needed after calibration.
        #[arg(long)]
        approve_exact_sdl_sequence: bool,
        /// Bounded input exercise duration in seconds.
        #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u64).range(1..=60))]
        seconds: u64,
        /// Maximum number of frames to decode.
        #[arg(long, default_value_t = 4_096, value_parser = parse_frame_limit)]
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
    /// Exercise decoded BEE-021 wired input without retaining raw reports.
    #[cfg(windows)]
    UsbDecodedInputTest {
        /// Bounded input exercise duration in seconds.
        #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u64).range(1..=60))]
        seconds: u64,
        /// Maximum number of frames to decode.
        #[arg(long, default_value_t = 4_096, value_parser = parse_frame_limit)]
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
        | Command::UsbObserve { .. }
        | Command::UsbDecodedInputTest { .. }
        | Command::UsbCalibrationStatus { .. }
        | Command::UsbCalibratedInputTest { .. } => "windows_usb_read_only",
        #[cfg(windows)]
        Command::UsbInputProbe { .. }
        | Command::UsbReport5InputProbe { .. }
        | Command::UsbDescribedInputProbe { .. }
        | Command::UsbSdlReferenceProbe { .. }
        | Command::UsbMotionEnableProbe { .. } => "windows_usb_reviewed_experiment",
        #[cfg(windows)]
        Command::BluetoothInventory
        | Command::BluetoothScan { .. }
        | Command::BluetoothLabStatus
        | Command::BluetoothPairtoolScan { .. } => "windows_bluetooth_diagnostic",
        #[cfg(windows)]
        Command::BleScan { .. }
        | Command::BleDeviceScan { .. }
        | Command::BleAdapterStatus
        | Command::PackageStatus => "windows_ble_read_only",
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

#[cfg(windows)]
fn parse_frame_limit(value: &str) -> Result<usize, String> {
    let limit = value
        .parse::<usize>()
        .map_err(|_| "frame limit must be an integer".to_owned())?;
    if (1..=MAX_FRAME_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(format!(
            "frame limit must be between 1 and {MAX_FRAME_LIMIT}"
        ))
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
    BluetoothInventory {
        adapter_present: bool,
        devices: Vec<BluetoothDeviceView>,
    },
    #[cfg(windows)]
    BluetoothScan {
        seconds: u64,
        devices: Vec<BluetoothDeviceView>,
    },
    #[cfg(windows)]
    BluetoothLabStatus {
        pairtool_available: bool,
        classic_bluetooth_available: bool,
    },
    #[cfg(windows)]
    BluetoothPairtoolScan {
        seconds: u64,
        endpoint_digests: Vec<String>,
    },
    #[cfg(windows)]
    BleScan {
        seconds: u64,
        advertisements: Vec<BleAdvertisementView>,
    },
    #[cfg(windows)]
    BleDeviceScan {
        seconds: u64,
        devices: Vec<BleDeviceView>,
    },
    #[cfg(windows)]
    PackageStatus {
        package_identity_present: bool,
    },
    #[cfg(windows)]
    BleAdapterStatus {
        low_energy_supported: bool,
        central_role_supported: bool,
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
        command_reply_lengths: Vec<usize>,
        reports: Vec<UsbInputMetadataView>,
    },
    #[cfg(windows)]
    UsbDecodedInput {
        buttons_seen: Vec<String>,
        axis_ranges: Vec<AxisRangeView>,
        frames: usize,
        motion_samples: usize,
        acceleration_ranges: Vec<MotionRangeView>,
        angular_velocity_ranges: Vec<MotionRangeView>,
    },
    #[cfg(windows)]
    UsbCalibration {
        blocks_read: u8,
        factory_valid: bool,
        left_user_override: bool,
        right_user_override: bool,
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
struct BluetoothDeviceView {
    id_digest: String,
    name: Option<String>,
    paired: bool,
    enabled: bool,
}

#[cfg(windows)]
#[derive(Debug, Serialize)]
struct BleAdvertisementView {
    identifier_digest: String,
    local_name: Option<String>,
    switch2_service_advertised: bool,
}

#[cfg(windows)]
#[derive(Debug, Serialize)]
struct BleDeviceView {
    identifier_digest: String,
    local_name: Option<String>,
}

#[cfg(windows)]
#[derive(Debug, Serialize)]
struct UsbInputMetadataView {
    report_id: String,
    length: usize,
    count: usize,
}

#[cfg(windows)]
#[derive(Debug, Serialize)]
struct AxisRangeView {
    axis: String,
    minimum: i16,
    maximum: i16,
}

#[cfg(windows)]
#[derive(Debug, Serialize)]
struct MotionRangeView {
    axis: &'static str,
    minimum: f32,
    maximum: f32,
}

#[allow(clippy::too_many_lines)]
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
        Command::UsbInventory => usb_inventory(),
        #[cfg(windows)]
        Command::BluetoothInventory => bluetooth_inventory(),
        #[cfg(windows)]
        Command::BluetoothScan { seconds } => bluetooth_scan(seconds),
        #[cfg(windows)]
        Command::BluetoothLabStatus => bluetooth_lab_status(),
        #[cfg(windows)]
        Command::BluetoothPairtoolScan {
            approve_active_discovery,
            seconds,
        } => bluetooth_pairtool_scan(approve_active_discovery, seconds),
        #[cfg(windows)]
        Command::BleScan { seconds } => ble_scan(seconds),
        #[cfg(windows)]
        Command::BleDeviceScan { seconds } => ble_device_scan(seconds),
        #[cfg(windows)]
        Command::PackageStatus => Ok(package_status()),
        #[cfg(windows)]
        Command::BleAdapterStatus => ble_adapter_status(),
        #[cfg(windows)]
        Command::UsbBulkInventory => usb_bulk_inventory(),
        #[cfg(windows)]
        Command::UsbInputProbe {
            approve_reviewed_write,
            seconds,
            limit,
        } => usb_input_probe(approve_reviewed_write, seconds, limit),
        #[cfg(windows)]
        Command::UsbReport5InputProbe {
            approve_reviewed_writes,
            seconds,
            limit,
        } => usb_report5_input_probe(approve_reviewed_writes, seconds, limit),
        #[cfg(windows)]
        Command::UsbDescribedInputProbe {
            approve_reviewed_writes,
            seconds,
            limit,
        } => usb_described_input_probe(approve_reviewed_writes, seconds, limit),
        #[cfg(windows)]
        Command::UsbSdlReferenceProbe {
            approve_exact_sdl_sequence,
            seconds,
            limit,
        } => usb_sdl_reference_probe(approve_exact_sdl_sequence, seconds, limit),
        #[cfg(windows)]
        Command::UsbMotionEnableProbe {
            approve_reviewed_write,
        } => usb_motion_enable_probe(approve_reviewed_write),
        #[cfg(windows)]
        Command::UsbCalibrationStatus {
            approve_read_only_calibration,
        } => usb_calibration_status(approve_read_only_calibration),
        #[cfg(windows)]
        command @ Command::UsbCalibratedInputTest { .. } => usb_calibrated_input_command(&command),
        #[cfg(windows)]
        Command::UsbDescriptor => usb_descriptor(),
        #[cfg(windows)]
        Command::UsbObserve { seconds, limit } => usb_observe(seconds, limit),
        #[cfg(windows)]
        Command::UsbDecodedInputTest { seconds, limit } => usb_decoded_input(seconds, limit),
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
fn usb_inventory() -> Result<Payload, UserSafeError> {
    Ok(Payload::UsbInterfaces {
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
    })
}

#[cfg(windows)]
fn bluetooth_inventory() -> Result<Payload, UserSafeError> {
    let inventory = crate::platform::windows::enumerate_bluetooth()?;
    Ok(Payload::BluetoothInventory {
        adapter_present: inventory.adapter_present,
        devices: inventory
            .devices
            .into_iter()
            .map(|device| BluetoothDeviceView {
                id_digest: device.id_digest,
                name: device.name,
                paired: device.paired,
                enabled: device.enabled,
            })
            .collect(),
    })
}

#[cfg(windows)]
fn bluetooth_scan(seconds: u64) -> Result<Payload, UserSafeError> {
    let scan = crate::platform::windows::scan_unpaired_bluetooth(Duration::from_secs(seconds))?;
    Ok(Payload::BluetoothScan {
        seconds: scan.duration.as_secs(),
        devices: scan
            .devices
            .into_iter()
            .map(|device| BluetoothDeviceView {
                id_digest: device.id_digest,
                name: device.name,
                paired: device.paired,
                enabled: device.enabled,
            })
            .collect(),
    })
}

#[cfg(windows)]
fn bluetooth_lab_status() -> Result<Payload, UserSafeError> {
    let status = crate::platform::windows::inspect_pairtool()?;
    Ok(Payload::BluetoothLabStatus {
        pairtool_available: status.available,
        classic_bluetooth_available: status.classic_bluetooth_available,
    })
}

#[cfg(windows)]
fn bluetooth_pairtool_scan(approved: bool, seconds: u64) -> Result<Payload, UserSafeError> {
    if !approved {
        return Err(UserSafeError::new(
            ErrorCategory::PermissionDenied,
            "active Bluetooth discovery requires --approve-active-discovery",
        ));
    }
    let discovery = crate::platform::windows::discover_with_pairtool(Duration::from_secs(seconds))?;
    Ok(Payload::BluetoothPairtoolScan {
        seconds: discovery.duration.as_secs(),
        endpoint_digests: discovery.endpoint_digests,
    })
}

#[cfg(windows)]
fn ble_scan(seconds: u64) -> Result<Payload, UserSafeError> {
    let scan = crate::platform::windows::scan_ble_advertisements(Duration::from_secs(seconds))?;
    Ok(Payload::BleScan {
        seconds: scan.duration.as_secs(),
        advertisements: scan
            .advertisements
            .into_iter()
            .map(|advertisement| BleAdvertisementView {
                identifier_digest: advertisement.identifier_digest,
                local_name: advertisement.local_name,
                switch2_service_advertised: advertisement.switch2_service_advertised,
            })
            .collect(),
    })
}

#[cfg(windows)]
fn ble_device_scan(seconds: u64) -> Result<Payload, UserSafeError> {
    let scan = crate::platform::windows::scan_unpaired_ble_devices(Duration::from_secs(seconds))?;
    Ok(Payload::BleDeviceScan {
        seconds: scan.duration.as_secs(),
        devices: scan
            .devices
            .into_iter()
            .map(|device| BleDeviceView {
                identifier_digest: device.identifier_digest,
                local_name: device.local_name,
            })
            .collect(),
    })
}

#[cfg(windows)]
fn ble_adapter_status() -> Result<Payload, UserSafeError> {
    let capabilities = crate::platform::windows::inspect_ble_adapter()?;
    Ok(Payload::BleAdapterStatus {
        low_energy_supported: capabilities.low_energy_supported,
        central_role_supported: capabilities.central_role_supported,
    })
}

#[cfg(windows)]
fn package_status() -> Payload {
    Payload::PackageStatus {
        package_identity_present: crate::platform::windows::has_package_identity(),
    }
}

#[cfg(windows)]
fn usb_descriptor() -> Result<Payload, UserSafeError> {
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
    Ok(usb_probe_payload(observation))
}

#[cfg(windows)]
fn usb_report5_input_probe(
    approved: bool,
    seconds: u64,
    limit: usize,
) -> Result<Payload, UserSafeError> {
    if !approved {
        return Err(UserSafeError::new(
            ErrorCategory::PermissionDenied,
            "the two reviewed USB writes require --approve-reviewed-writes",
        ));
    }
    let observation = crate::platform::windows::run_report5_input_probe(
        NINTENDO_VENDOR_ID,
        BEE_021_USB_PRODUCT_ID,
        BEE_021_BULK_INTERFACE,
        Duration::from_secs(seconds),
        limit,
    )?;
    Ok(usb_probe_payload(observation))
}

#[cfg(windows)]
fn usb_described_input_probe(
    approved: bool,
    seconds: u64,
    limit: usize,
) -> Result<Payload, UserSafeError> {
    if !approved {
        return Err(UserSafeError::new(
            ErrorCategory::PermissionDenied,
            "the four reviewed USB writes require --approve-reviewed-writes",
        ));
    }
    let observation = crate::platform::windows::run_described_input_probe(
        NINTENDO_VENDOR_ID,
        BEE_021_USB_PRODUCT_ID,
        BEE_021_BULK_INTERFACE,
        Duration::from_secs(seconds),
        limit,
    )?;
    Ok(usb_probe_payload(observation))
}

#[cfg(windows)]
fn usb_sdl_reference_probe(
    approved: bool,
    seconds: u64,
    limit: usize,
) -> Result<Payload, UserSafeError> {
    if !approved {
        return Err(UserSafeError::new(
            ErrorCategory::PermissionDenied,
            "the exact SDL sequence requires --approve-exact-sdl-sequence",
        ));
    }
    let observation = crate::platform::windows::run_sdl_reference_input_probe(
        NINTENDO_VENDOR_ID,
        BEE_021_USB_PRODUCT_ID,
        BEE_021_BULK_INTERFACE,
        Duration::from_secs(seconds),
        limit,
    )?;
    Ok(usb_probe_payload(observation))
}

#[cfg(windows)]
fn usb_motion_enable_probe(approved: bool) -> Result<Payload, UserSafeError> {
    if !approved {
        return Err(UserSafeError::new(
            ErrorCategory::PermissionDenied,
            "the reviewed motion write requires --approve-reviewed-write",
        ));
    }
    let observation = crate::platform::windows::run_motion_enable_probe(
        NINTENDO_VENDOR_ID,
        BEE_021_USB_PRODUCT_ID,
        BEE_021_BULK_INTERFACE,
    )?;
    Ok(usb_probe_payload(observation))
}

#[cfg(windows)]
fn usb_calibration_status(approved: bool) -> Result<Payload, UserSafeError> {
    if !approved {
        return Err(UserSafeError::new(
            ErrorCategory::PermissionDenied,
            "read-only calibration requires --approve-read-only-calibration",
        ));
    }
    let observation = crate::platform::windows::read_calibration(
        NINTENDO_VENDOR_ID,
        BEE_021_USB_PRODUCT_ID,
        BEE_021_BULK_INTERFACE,
    )?;
    Ok(Payload::UsbCalibration {
        blocks_read: observation.blocks_read,
        factory_valid: observation.status.factory_valid,
        left_user_override: observation.status.left_user_override,
        right_user_override: observation.status.right_user_override,
    })
}

#[cfg(windows)]
fn usb_calibrated_input(
    approved_calibration: bool,
    approved_sequence: bool,
    seconds: u64,
    limit: usize,
) -> Result<Payload, UserSafeError> {
    if !approved_calibration {
        return Err(UserSafeError::new(
            ErrorCategory::PermissionDenied,
            "calibrated input requires --approve-read-only-calibration",
        ));
    }
    if !approved_sequence {
        return Err(UserSafeError::new(
            ErrorCategory::PermissionDenied,
            "calibrated input requires --approve-exact-sdl-sequence",
        ));
    }
    let observation = crate::platform::windows::observe_calibrated_usb_input(
        NINTENDO_VENDOR_ID,
        BEE_021_USB_PRODUCT_ID,
        Duration::from_secs(seconds),
        limit,
    )?;
    Ok(decoded_input_payload(observation))
}

#[cfg(windows)]
fn usb_calibrated_input_command(command: &Command) -> Result<Payload, UserSafeError> {
    let &Command::UsbCalibratedInputTest {
        approve_read_only_calibration,
        approve_exact_sdl_sequence,
        seconds,
        limit,
    } = command
    else {
        unreachable!("caller matches calibrated input command")
    };
    usb_calibrated_input(
        approve_read_only_calibration,
        approve_exact_sdl_sequence,
        seconds,
        limit,
    )
}

#[cfg(windows)]
fn usb_probe_payload(
    observation: crate::platform::windows::MinimalInputProbeObservation,
) -> Payload {
    Payload::UsbInputProbe {
        command_reply_lengths: observation.command_reply_lengths,
        reports: observation
            .reports
            .into_iter()
            .map(|report| UsbInputMetadataView {
                report_id: format!("{:02x}", report.report_id),
                length: report.length,
                count: 1,
            })
            .collect(),
    }
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

#[cfg(windows)]
fn usb_decoded_input(seconds: u64, limit: usize) -> Result<Payload, UserSafeError> {
    let observation = crate::platform::windows::observe_decoded_usb_input(
        NINTENDO_VENDOR_ID,
        BEE_021_USB_PRODUCT_ID,
        Duration::from_secs(seconds),
        limit,
    )?;
    Ok(decoded_input_payload(observation))
}

#[cfg(windows)]
fn decoded_input_payload(
    observation: crate::platform::windows::UsbDecodedInputObservation,
) -> Payload {
    Payload::UsbDecodedInput {
        buttons_seen: observation
            .buttons_seen
            .into_iter()
            .map(|button| format!("{button:?}"))
            .collect(),
        axis_ranges: observation
            .axis_ranges
            .into_iter()
            .map(|(axis, (minimum, maximum))| AxisRangeView {
                axis: format!("{axis:?}"),
                minimum,
                maximum,
            })
            .collect(),
        frames: observation.frames,
        motion_samples: observation.motion_samples,
        acceleration_ranges: motion_range_views(observation.acceleration_ranges),
        angular_velocity_ranges: motion_range_views(observation.angular_velocity_ranges),
    }
}

#[cfg(windows)]
fn motion_range_views(ranges: Option<[(f32, f32); 3]>) -> Vec<MotionRangeView> {
    let Some(ranges) = ranges else {
        return Vec::new();
    };
    ["x", "y", "z"]
        .into_iter()
        .zip(ranges)
        .map(|(axis, (minimum, maximum))| MotionRangeView {
            axis,
            minimum,
            maximum,
        })
        .collect()
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

#[allow(clippy::too_many_lines)]
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
        Payload::UsbInterfaces { items } => render_usb_interfaces(&mut output, items),
        #[cfg(windows)]
        Payload::BluetoothInventory {
            adapter_present,
            devices,
        } => render_bluetooth_inventory(&mut output, adapter_present, devices),
        #[cfg(windows)]
        Payload::BluetoothScan { seconds, devices } => {
            render_bluetooth_scan(&mut output, seconds, devices);
        }
        #[cfg(windows)]
        Payload::BluetoothLabStatus {
            pairtool_available,
            classic_bluetooth_available,
        } => render_bluetooth_lab_status(
            &mut output,
            pairtool_available,
            classic_bluetooth_available,
        ),
        #[cfg(windows)]
        Payload::BluetoothPairtoolScan {
            seconds,
            endpoint_digests,
        } => render_bluetooth_pairtool_scan(&mut output, seconds, endpoint_digests),
        #[cfg(windows)]
        Payload::BleScan {
            seconds,
            advertisements,
        } => render_ble_scan(&mut output, seconds, advertisements),
        #[cfg(windows)]
        Payload::BleDeviceScan { seconds, devices } => {
            render_ble_device_scan(&mut output, seconds, devices);
        }
        #[cfg(windows)]
        Payload::PackageStatus {
            package_identity_present,
        } => {
            let _ = writeln!(
                output,
                "Windows package identity: {package_identity_present}"
            );
        }
        #[cfg(windows)]
        Payload::BleAdapterStatus {
            low_energy_supported,
            central_role_supported,
        } => {
            let _ = writeln!(
                output,
                "BLE adapter: low_energy_supported={low_energy_supported} central_role_supported={central_role_supported}"
            );
        }
        #[cfg(windows)]
        Payload::UsbBulkInterface {
            interface_number,
            input_endpoint,
            output_endpoint,
            input_max_packet_size,
            output_max_packet_size,
        } => render_usb_bulk_interface(
            &mut output,
            interface_number,
            &input_endpoint,
            &output_endpoint,
            input_max_packet_size,
            output_max_packet_size,
        ),
        #[cfg(windows)]
        Payload::UsbDescriptor { length, sha256 } => {
            render_usb_descriptor(&mut output, length, &sha256);
        }
        #[cfg(windows)]
        Payload::UsbInputMetadata { items } => render_usb_input_metadata(&mut output, items),
        #[cfg(windows)]
        Payload::UsbInputProbe {
            command_reply_lengths,
            reports,
        } => render_usb_input_probe(&mut output, command_reply_lengths, reports),
        #[cfg(windows)]
        payload @ Payload::UsbDecodedInput { .. } => {
            render_usb_decoded_payload(&mut output, payload);
        }
        #[cfg(windows)]
        Payload::UsbCalibration {
            blocks_read,
            factory_valid,
            left_user_override,
            right_user_override,
        } => render_usb_calibration(
            &mut output,
            blocks_read,
            factory_valid,
            left_user_override,
            right_user_override,
        ),
    }
    output
}

#[cfg(windows)]
fn render_usb_decoded_payload(output: &mut String, payload: Payload) {
    let Payload::UsbDecodedInput {
        buttons_seen,
        axis_ranges,
        frames,
        motion_samples,
        acceleration_ranges,
        angular_velocity_ranges,
    } = payload
    else {
        unreachable!("caller matches the decoded input payload")
    };
    render_usb_decoded_input(
        output,
        &buttons_seen,
        axis_ranges,
        frames,
        motion_samples,
        acceleration_ranges,
        angular_velocity_ranges,
    );
}

#[cfg(windows)]
fn render_usb_calibration(
    output: &mut String,
    blocks_read: u8,
    factory_valid: bool,
    left_user_override: bool,
    right_user_override: bool,
) {
    let _ = writeln!(
        output,
        "calibration: blocks={blocks_read} factory_valid={factory_valid} left_user_override={left_user_override} right_user_override={right_user_override}"
    );
}

#[cfg(windows)]
fn render_usb_interfaces(output: &mut String, items: Vec<UsbInterfaceView>) {
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
fn render_bluetooth_inventory(
    output: &mut String,
    adapter_present: bool,
    devices: Vec<BluetoothDeviceView>,
) {
    let _ = writeln!(output, "bluetooth adapter present: {adapter_present}");
    for device in devices {
        let name = device.name.as_deref().unwrap_or("unnamed device");
        let _ = writeln!(
            output,
            "{name}: id={} paired={} enabled={}",
            device.id_digest, device.paired, device.enabled
        );
    }
}

#[cfg(windows)]
fn render_bluetooth_scan(output: &mut String, seconds: u64, devices: Vec<BluetoothDeviceView>) {
    let _ = writeln!(output, "bluetooth scan: {seconds} seconds");
    for device in devices {
        let name = device.name.as_deref().unwrap_or("unnamed device");
        let _ = writeln!(
            output,
            "{name}: id={} paired={} enabled={}",
            device.id_digest, device.paired, device.enabled
        );
    }
}

#[cfg(windows)]
fn render_bluetooth_lab_status(
    output: &mut String,
    pairtool_available: bool,
    classic_bluetooth_available: bool,
) {
    let _ = writeln!(
        output,
        "pairtool available: {pairtool_available} classic_bluetooth_available: {classic_bluetooth_available}"
    );
}

#[cfg(windows)]
fn render_bluetooth_pairtool_scan(
    output: &mut String,
    seconds: u64,
    endpoint_digests: Vec<String>,
) {
    let _ = writeln!(output, "pairtool active scan: {seconds} seconds");
    for endpoint_digest in endpoint_digests {
        let _ = writeln!(output, "discovered endpoint: {endpoint_digest}");
    }
}

#[cfg(windows)]
fn render_ble_scan(output: &mut String, seconds: u64, advertisements: Vec<BleAdvertisementView>) {
    let _ = writeln!(output, "BLE scan: {seconds} seconds");
    for advertisement in advertisements {
        let name = advertisement
            .local_name
            .as_deref()
            .unwrap_or("unnamed BLE device");
        let _ = writeln!(
            output,
            "{name}: id={} switch2_service_advertised={}",
            advertisement.identifier_digest, advertisement.switch2_service_advertised
        );
    }
}

#[cfg(windows)]
fn render_ble_device_scan(output: &mut String, seconds: u64, devices: Vec<BleDeviceView>) {
    let _ = writeln!(output, "BLE device-selector scan: {seconds} seconds");
    if devices.is_empty() {
        let _ = writeln!(output, "no BLE devices observed");
    }
    for device in devices {
        let _ = writeln!(
            output,
            "{}: id={}",
            device.local_name.as_deref().unwrap_or("unnamed BLE device"),
            device.identifier_digest
        );
    }
}

#[cfg(windows)]
fn render_usb_bulk_interface(
    output: &mut String,
    interface_number: u8,
    input_endpoint: &str,
    output_endpoint: &str,
    input_max_packet_size: usize,
    output_max_packet_size: usize,
) {
    let _ = writeln!(
        output,
        "interface {interface_number}: bulk-in=0x{input_endpoint} ({input_max_packet_size} bytes) bulk-out=0x{output_endpoint} ({output_max_packet_size} bytes)"
    );
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
    command_reply_lengths: Vec<usize>,
    reports: Vec<UsbInputMetadataView>,
) {
    for (index, length) in command_reply_lengths.into_iter().enumerate() {
        let _ = writeln!(output, "command {index} reply: {length} bytes");
    }
    for report in reports {
        let _ = writeln!(
            output,
            "report 0x{}: {} bytes",
            report.report_id, report.length
        );
    }
}

#[cfg(windows)]
fn render_usb_decoded_input(
    output: &mut String,
    buttons_seen: &[String],
    axis_ranges: Vec<AxisRangeView>,
    frames: usize,
    motion_samples: usize,
    acceleration_ranges: Vec<MotionRangeView>,
    angular_velocity_ranges: Vec<MotionRangeView>,
) {
    let _ = writeln!(output, "frames: {frames}");
    let _ = writeln!(output, "buttons seen: {}", buttons_seen.join(", "));
    for range in axis_ranges {
        let _ = writeln!(
            output,
            "{}: {}..{}",
            range.axis, range.minimum, range.maximum
        );
    }
    let _ = writeln!(output, "motion samples: {motion_samples}");
    render_motion_ranges(output, "acceleration", acceleration_ranges);
    render_motion_ranges(output, "angular velocity", angular_velocity_ranges);
}

#[cfg(windows)]
fn render_motion_ranges(output: &mut String, label: &str, ranges: Vec<MotionRangeView>) {
    for range in ranges {
        let _ = writeln!(
            output,
            "{label} {}: {:.3}..{:.3}",
            range.axis, range.minimum, range.maximum
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
            result_file: None,
            command: Command::Scan,
        });
        assert_eq!(human.exit_code, 0);
        assert!(human.output.contains("BEE-021 simulated controller"));

        let json = run(Args {
            json: true,
            timeout: 1,
            result_file: None,
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
            result_file: None,
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
            result_file: None,
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
