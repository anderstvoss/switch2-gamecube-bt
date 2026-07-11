//! Bounded raw controller reports.

/// Maximum accepted HID report size.
pub const MAX_REPORT_SIZE: usize = 4_096;

/// Physical transport that produced a report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Transport {
    /// USB HID transport.
    Usb,
    /// Bluetooth HID transport.
    Bluetooth,
}

/// Validation failure for a raw report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReportError {
    /// Reports must contain at least a report identifier.
    Empty,
    /// The report exceeded [`MAX_REPORT_SIZE`].
    TooLarge,
}

/// An owned, bounded HID report that preserves unknown data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawReport {
    transport: Transport,
    bytes: Box<[u8]>,
}

impl RawReport {
    /// Validates and stores a raw report.
    ///
    /// # Errors
    ///
    /// Returns [`ReportError`] when the report is empty or exceeds the fixed
    /// safety bound.
    pub fn new(transport: Transport, bytes: impl Into<Box<[u8]>>) -> Result<Self, ReportError> {
        let bytes = bytes.into();
        if bytes.is_empty() {
            return Err(ReportError::Empty);
        }
        if bytes.len() > MAX_REPORT_SIZE {
            return Err(ReportError::TooLarge);
        }
        Ok(Self { transport, bytes })
    }

    /// Returns the source transport.
    #[must_use]
    pub const fn transport(&self) -> Transport {
        self.transport
    }

    /// Returns the unmodified report bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the first byte as the report identifier.
    #[must_use]
    pub fn report_id(&self) -> u8 {
        self.bytes[0]
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_REPORT_SIZE, RawReport, ReportError, Transport};

    #[test]
    fn preserves_unknown_reports() {
        let report =
            RawReport::new(Transport::Bluetooth, [0x31, 0xAA].as_slice()).expect("bounded report");
        assert_eq!(report.report_id(), 0x31);
        assert_eq!(report.bytes(), [0x31, 0xAA]);
    }

    #[test]
    fn rejects_empty_and_oversized_reports() {
        assert_eq!(
            RawReport::new(Transport::Usb, &[][..]),
            Err(ReportError::Empty)
        );
        assert_eq!(
            RawReport::new(Transport::Usb, vec![0; MAX_REPORT_SIZE + 1]),
            Err(ReportError::TooLarge)
        );
    }
}
