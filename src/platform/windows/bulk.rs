//! Read-only Windows USB bulk-interface inventory.

use nusb::{MaybeFuture, descriptors::TransferType, transfer::Direction};

use crate::domain::{ErrorCategory, UserSafeError};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EndpointCandidate {
    address: u8,
    direction: Direction,
    transfer_type: TransferType,
    max_packet_size: usize,
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
}
