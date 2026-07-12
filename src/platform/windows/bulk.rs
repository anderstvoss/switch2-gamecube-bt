//! Read-only Windows USB bulk-interface inventory and bounded transport seam.

use std::time::Duration;
use std::time::Instant;

use nusb::{
    Endpoint, Interface, MaybeFuture,
    descriptors::TransferType,
    transfer::{Buffer, Bulk, Direction, In, Out, TransferError},
};

use crate::{
    controllers::bee021::usb_protocol::{BulkTransport, ClassifiedCommand, TransportError},
    domain::{ErrorCategory, UserSafeError},
};

const MAX_TRANSFER_LENGTH: usize = 64;
const ENDPOINT_DIRECTION_MASK: u8 = 0x80;

/// Sanitized layout of a USB interface's bulk endpoints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BulkEndpointLayout {
    /// USB interface number containing the endpoints.
    pub interface_number: u8,
    /// Device-to-host bulk endpoint address.
    pub input_endpoint: u8,
    /// Host-to-device bulk endpoint address.
    pub output_endpoint: u8,
    /// Maximum packet size of the input endpoint.
    pub input_max_packet_size: usize,
    /// Maximum packet size of the output endpoint.
    pub output_max_packet_size: usize,
}

/// Sanitized metadata for one bulk input report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BulkReportObservation {
    /// First report byte, interpreted as the report identifier.
    pub report_id: u8,
    /// Number of bytes received.
    pub length: usize,
}

/// Sanitized outcome of the explicitly approved start-stream probe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinimalInputProbeObservation {
    /// Length of each command reply in command order.
    pub command_reply_lengths: Vec<usize>,
    /// Bounded report metadata observed after the reply.
    pub reports: Vec<BulkReportObservation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EndpointCandidate {
    address: u8,
    direction: Direction,
    transfer_type: TransferType,
    max_packet_size: usize,
}

/// Internal live bulk transport with fixed bounds and deadlines.
///
/// The interface field keeps the exclusive claim alive for the lifetime of
/// both endpoints. This type is deliberately not exported from the Windows
/// platform module and has no CLI construction path.
pub(super) struct WinUsbBulkTransport {
    _interface: Interface,
    input: Endpoint<Bulk, In>,
    output: Endpoint<Bulk, Out>,
    timeout: Duration,
}

impl WinUsbBulkTransport {
    fn receive_report_metadata(&mut self) -> Result<Option<BulkReportObservation>, TransportError> {
        let completed = match self
            .input
            .transfer_blocking(Buffer::new(MAX_TRANSFER_LENGTH), self.timeout)
            .into_result()
        {
            Ok(completed) => completed,
            Err(TransferError::Cancelled) => return Ok(None),
            Err(error) => return Err(map_transfer_error(error)),
        };
        let Some(report_id) = completed.first().copied() else {
            return Err(TransportError::TransferFailed);
        };
        Ok(Some(BulkReportObservation {
            report_id,
            length: completed.len(),
        }))
    }
}

impl BulkTransport for WinUsbBulkTransport {
    fn send(&mut self, packet: &[u8]) -> Result<(), TransportError> {
        validate_transfer_length(packet.len())?;
        let completed = self
            .output
            .transfer_blocking(packet.to_vec().into(), self.timeout)
            .into_result()
            .map_err(map_transfer_error)?;
        if completed.len() != packet.len() {
            return Err(TransportError::TransferFailed);
        }
        Ok(())
    }

    fn receive(&mut self, maximum_length: usize) -> Result<usize, TransportError> {
        validate_transfer_length(maximum_length)?;
        // WinUSB bulk reads are requested in endpoint-sized units. The actual
        // reply must still satisfy the caller's stricter bound.
        let completed = self
            .input
            .transfer_blocking(Buffer::new(MAX_TRANSFER_LENGTH), self.timeout)
            .into_result()
            .map_err(map_transfer_error)?;
        if completed.is_empty() || completed.len() > maximum_length {
            return Err(TransportError::TransferFailed);
        }
        Ok(completed.len())
    }
}

/// Opens and claims the already-inspected bulk interface.
///
/// This factory is intentionally private to the Windows platform adapter and
/// is not currently called by production code. It must not be exposed through
/// the CLI before the live initialization safety gate is approved.
pub(super) fn open_bulk_transport(
    vendor_id: u16,
    product_id: u16,
    layout: BulkEndpointLayout,
    timeout: Duration,
) -> Result<WinUsbBulkTransport, UserSafeError> {
    validate_layout(layout, timeout)?;
    let devices = nusb::list_devices()
        .wait()
        .map_err(|_| platform_error("Windows USB device enumeration failed"))?;
    let mut matches = devices
        .filter(|device| device.vendor_id() == vendor_id && device.product_id() == product_id);
    let device_info = matches
        .next()
        .ok_or_else(|| platform_error("matching Windows USB device was not found"))?;
    if matches.next().is_some() {
        return Err(invalid_data(
            "multiple matching Windows USB devices require explicit selection",
        ));
    }
    let device = device_info
        .open()
        .wait()
        .map_err(|_| platform_error("Windows USB device could not be opened"))?;
    let interface = device
        .claim_interface(layout.interface_number)
        .wait()
        .map_err(|_| platform_error("Windows USB bulk interface could not be claimed"))?;
    let input = interface
        .endpoint::<Bulk, In>(layout.input_endpoint)
        .map_err(|_| invalid_data("USB bulk input endpoint could not be opened"))?;
    let output = interface
        .endpoint::<Bulk, Out>(layout.output_endpoint)
        .map_err(|_| invalid_data("USB bulk output endpoint could not be opened"))?;
    Ok(WinUsbBulkTransport {
        _interface: interface,
        input,
        output,
        timeout,
    })
}

/// Runs the explicitly approved one-packet BEE-021 start-stream experiment.
///
/// The function sends exactly one allowlisted command, records only sanitized
/// reply/report metadata, and drops the claimed interface on return. It never
/// sends calibration, firmware, reset, pairing, rumble, LED, or cleanup data.
///
/// # Errors
///
/// Returns a privacy-safe error if identity/endpoint validation, claiming, the
/// single command transfer, or bounded report observation fails.
pub fn run_minimal_input_probe(
    vendor_id: u16,
    product_id: u16,
    interface_number: u8,
    duration: Duration,
    report_limit: usize,
) -> Result<MinimalInputProbeObservation, UserSafeError> {
    run_input_probe(
        vendor_id,
        product_id,
        interface_number,
        duration,
        report_limit,
        &[ClassifiedCommand::StartInputStream],
    )
}

/// Runs the approved report-format `0x05` plus start-stream experiment.
///
/// # Errors
///
/// Returns a privacy-safe error under the same identity, endpoint, transfer,
/// and observation bounds as [`run_minimal_input_probe`].
pub fn run_report5_input_probe(
    vendor_id: u16,
    product_id: u16,
    interface_number: u8,
    duration: Duration,
    report_limit: usize,
) -> Result<MinimalInputProbeObservation, UserSafeError> {
    run_input_probe(
        vendor_id,
        product_id,
        interface_number,
        duration,
        report_limit,
        &[
            ClassifiedCommand::SetInputReportFormat,
            ClassifiedCommand::StartInputStream,
        ],
    )
}

/// Runs the approved four-command described, non-rumble input experiment.
///
/// # Errors
///
/// Returns a privacy-safe error under the same identity, endpoint, transfer,
/// and observation bounds as [`run_minimal_input_probe`].
pub fn run_described_input_probe(
    vendor_id: u16,
    product_id: u16,
    interface_number: u8,
    duration: Duration,
    report_limit: usize,
) -> Result<MinimalInputProbeObservation, UserSafeError> {
    run_input_probe(
        vendor_id,
        product_id,
        interface_number,
        duration,
        report_limit,
        &[
            ClassifiedCommand::SetFeatureOutputMask,
            ClassifiedCommand::EnableFeatureOutputChannels,
            ClassifiedCommand::SetInputReportFormat,
            ClassifiedCommand::StartInputStream,
        ],
    )
}

fn run_input_probe(
    vendor_id: u16,
    product_id: u16,
    interface_number: u8,
    duration: Duration,
    report_limit: usize,
    commands: &[ClassifiedCommand],
) -> Result<MinimalInputProbeObservation, UserSafeError> {
    if duration.is_zero() || report_limit == 0 {
        return Err(invalid_data("USB input probe bounds must be nonzero"));
    }
    if commands.is_empty() {
        return Err(invalid_data("USB input probe requires a reviewed command"));
    }
    let layout = inspect_bulk_endpoints(vendor_id, product_id, interface_number)?;
    let transfer_timeout = Duration::from_millis(250).min(duration);
    let mut transport = open_bulk_transport(vendor_id, product_id, layout, transfer_timeout)?;
    let mut command_reply_lengths = Vec::with_capacity(commands.len());
    for command in commands {
        transport.send(command.packet()).map_err(transport_error)?;
        command_reply_lengths.push(
            transport
                .receive(MAX_TRANSFER_LENGTH)
                .map_err(transport_error)?,
        );
    }

    let deadline = Instant::now() + duration;
    let mut reports = Vec::new();
    while reports.len() < report_limit && Instant::now() < deadline {
        if let Some(report) = transport
            .receive_report_metadata()
            .map_err(transport_error)?
        {
            reports.push(report);
        }
    }
    Ok(MinimalInputProbeObservation {
        command_reply_lengths,
        reports,
    })
}

fn validate_layout(layout: BulkEndpointLayout, timeout: Duration) -> Result<(), UserSafeError> {
    if timeout.is_zero() {
        return Err(invalid_data("USB bulk transfer timeout must be nonzero"));
    }
    if layout.input_endpoint & ENDPOINT_DIRECTION_MASK != Direction::In as u8
        || layout.output_endpoint & ENDPOINT_DIRECTION_MASK != Direction::Out as u8
    {
        return Err(invalid_data("USB bulk endpoint direction is invalid"));
    }
    if layout.input_max_packet_size == 0
        || layout.output_max_packet_size == 0
        || layout.input_max_packet_size > MAX_TRANSFER_LENGTH
        || layout.output_max_packet_size > MAX_TRANSFER_LENGTH
    {
        return Err(invalid_data("USB bulk endpoint packet size is unsupported"));
    }
    Ok(())
}

fn validate_transfer_length(length: usize) -> Result<(), TransportError> {
    if length == 0 || length > MAX_TRANSFER_LENGTH {
        Err(TransportError::TransferFailed)
    } else {
        Ok(())
    }
}

const fn map_transfer_error(error: TransferError) -> TransportError {
    match error {
        TransferError::Cancelled => TransportError::Timeout,
        TransferError::Stall
        | TransferError::Disconnected
        | TransferError::Fault
        | TransferError::InvalidArgument
        | TransferError::Unknown(_) => TransportError::TransferFailed,
    }
}

fn transport_error(error: TransportError) -> UserSafeError {
    match error {
        TransportError::Timeout => UserSafeError::new(
            ErrorCategory::Timeout,
            "Windows USB bulk transfer timed out",
        ),
        TransportError::TransferFailed => platform_error("Windows USB bulk transfer failed"),
    }
}

/// Reads the active USB configuration and identifies a single bulk endpoint
/// in each direction on `interface_number`.
///
/// This function opens only the device-level `WinUSB` handle needed to read the
/// active configuration descriptor. It does not claim an interface or perform
/// input, output, control, or feature transfers.
///
/// # Errors
///
/// Returns a privacy-safe error if device enumeration is ambiguous, the
/// configuration cannot be inspected, or the expected endpoint pair is absent.
pub fn inspect_bulk_endpoints(
    vendor_id: u16,
    product_id: u16,
    interface_number: u8,
) -> Result<BulkEndpointLayout, UserSafeError> {
    let devices = nusb::list_devices()
        .wait()
        .map_err(|_| platform_error("Windows USB device enumeration failed"))?;
    let mut matches = devices
        .filter(|device| device.vendor_id() == vendor_id && device.product_id() == product_id);
    let device_info = matches
        .next()
        .ok_or_else(|| platform_error("matching Windows USB device was not found"))?;
    if matches.next().is_some() {
        return Err(invalid_data(
            "multiple matching Windows USB devices require explicit selection",
        ));
    }
    if !device_info
        .interfaces()
        .any(|interface| interface.interface_number() == interface_number)
    {
        return Err(invalid_data("expected Windows USB interface was not found"));
    }

    let device = device_info
        .open()
        .wait()
        .map_err(|_| platform_error("Windows USB device could not be opened for inspection"))?;
    let configuration = device
        .active_configuration()
        .map_err(|_| platform_error("active USB configuration could not be read"))?;
    let interface = configuration
        .interfaces()
        .find(|interface| interface.interface_number() == interface_number)
        .ok_or_else(|| invalid_data("expected USB interface descriptor was not found"))?
        .first_alt_setting();
    let endpoints = interface.endpoints().map(|endpoint| EndpointCandidate {
        address: endpoint.address(),
        direction: endpoint.direction(),
        transfer_type: endpoint.transfer_type(),
        max_packet_size: endpoint.max_packet_size(),
    });

    select_bulk_endpoints(interface_number, endpoints)
}

fn select_bulk_endpoints(
    interface_number: u8,
    endpoints: impl IntoIterator<Item = EndpointCandidate>,
) -> Result<BulkEndpointLayout, UserSafeError> {
    let mut input = None;
    let mut output = None;
    for endpoint in endpoints
        .into_iter()
        .filter(|endpoint| endpoint.transfer_type == TransferType::Bulk)
    {
        let slot = match endpoint.direction {
            Direction::In => &mut input,
            Direction::Out => &mut output,
        };
        if slot.replace(endpoint).is_some() {
            return Err(invalid_data(
                "USB interface has multiple bulk endpoints in one direction",
            ));
        }
    }
    let input = input.ok_or_else(|| invalid_data("USB bulk input endpoint was not found"))?;
    let output = output.ok_or_else(|| invalid_data("USB bulk output endpoint was not found"))?;
    Ok(BulkEndpointLayout {
        interface_number,
        input_endpoint: input.address,
        output_endpoint: output.address,
        input_max_packet_size: input.max_packet_size,
        output_max_packet_size: output.max_packet_size,
    })
}

fn platform_error(message: &'static str) -> UserSafeError {
    UserSafeError::new(ErrorCategory::Platform, message)
}

fn invalid_data(message: &'static str) -> UserSafeError {
    UserSafeError::new(ErrorCategory::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(
        address: u8,
        direction: Direction,
        transfer_type: TransferType,
    ) -> EndpointCandidate {
        EndpointCandidate {
            address,
            direction,
            transfer_type,
            max_packet_size: 64,
        }
    }

    #[test]
    fn selects_one_bulk_endpoint_in_each_direction() {
        let layout = select_bulk_endpoints(
            1,
            [
                endpoint(0x81, Direction::In, TransferType::Bulk),
                endpoint(0x02, Direction::Out, TransferType::Bulk),
                endpoint(0x83, Direction::In, TransferType::Interrupt),
            ],
        )
        .expect("valid endpoint pair");
        assert_eq!(layout.input_endpoint, 0x81);
        assert_eq!(layout.output_endpoint, 0x02);
        assert_eq!(layout.input_max_packet_size, 64);
    }

    #[test]
    fn rejects_missing_or_ambiguous_endpoint_pairs() {
        assert!(select_bulk_endpoints(1, []).is_err());
        assert!(
            select_bulk_endpoints(
                1,
                [
                    endpoint(0x81, Direction::In, TransferType::Bulk),
                    endpoint(0x82, Direction::In, TransferType::Bulk),
                    endpoint(0x02, Direction::Out, TransferType::Bulk),
                ],
            )
            .is_err()
        );
    }

    #[test]
    fn transport_bounds_reject_empty_and_oversized_requests() {
        assert_eq!(
            validate_transfer_length(0),
            Err(TransportError::TransferFailed)
        );
        assert_eq!(validate_transfer_length(64), Ok(()));
        assert_eq!(
            validate_transfer_length(65),
            Err(TransportError::TransferFailed)
        );
    }

    #[test]
    fn live_layout_requires_directions_packet_bounds_and_deadline() {
        let valid = BulkEndpointLayout {
            interface_number: 1,
            input_endpoint: 0x82,
            output_endpoint: 0x02,
            input_max_packet_size: 64,
            output_max_packet_size: 64,
        };
        assert!(validate_layout(valid, Duration::from_millis(500)).is_ok());
        assert!(validate_layout(valid, Duration::ZERO).is_err());
        assert!(
            validate_layout(
                BulkEndpointLayout {
                    input_endpoint: 0x02,
                    ..valid
                },
                Duration::from_millis(500)
            )
            .is_err()
        );
        assert!(
            validate_layout(
                BulkEndpointLayout {
                    output_max_packet_size: 65,
                    ..valid
                },
                Duration::from_millis(500)
            )
            .is_err()
        );
    }

    #[test]
    fn transfer_failures_are_redacted_to_stable_categories() {
        assert_eq!(
            map_transfer_error(TransferError::Cancelled),
            TransportError::Timeout
        );
        assert_eq!(
            map_transfer_error(TransferError::Disconnected),
            TransportError::TransferFailed
        );
        assert_eq!(
            map_transfer_error(TransferError::Unknown(1234)),
            TransportError::TransferFailed
        );
    }
}
