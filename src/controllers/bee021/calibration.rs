//! Read-only BEE-021 calibration parsing.

/// One calibrated 12-bit axis.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AxisCalibration {
    neutral: u16,
    positive_span: u16,
    negative_span: u16,
}

impl AxisCalibration {
    /// Normalizes one raw 12-bit value around its factory or user center.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "value is clamped to i16 range"
    )]
    pub fn normalize(self, value: u16) -> i16 {
        let (offset, extent) = if value >= self.neutral {
            (
                i32::from(value - self.neutral),
                i32::from(self.positive_span),
            )
        } else {
            (
                i32::from(value) - i32::from(self.neutral),
                i32::from(self.negative_span),
            )
        };
        (offset * 32_767 / extent).clamp(-32_767, 32_767) as i16
    }
}

/// Calibrated pair of stick axes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct StickCalibration {
    /// Horizontal axis calibration.
    pub x: AxisCalibration,
    /// Vertical axis calibration.
    pub y: AxisCalibration,
}

/// Read-only BEE-021 calibration used locally by the decoder.
#[derive(Clone, PartialEq)]
pub struct Bee021Calibration {
    /// Main stick calibration, with a validated user override when present.
    pub left_stick: StickCalibration,
    /// C-stick calibration, with a validated user override when present.
    pub right_stick: StickCalibration,
    /// Main-trigger zero point.
    pub left_trigger_zero: u8,
    /// C-trigger zero point.
    pub right_trigger_zero: u8,
    /// Gyroscope bias in SDL axis order.
    pub gyro_bias: [f32; 3],
    /// Accelerometer bias in SDL axis order.
    pub acceleration_bias: [f32; 3],
}

/// Sanitized parse status for the read-only calibration operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalibrationStatus {
    /// Factory calibration blocks parsed successfully.
    pub factory_valid: bool,
    /// A valid user override replaced the main-stick calibration.
    pub left_user_override: bool,
    /// A valid user override replaced the C-stick calibration.
    pub right_user_override: bool,
}

/// Calibration parsing failure without block contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationError {
    /// A required 64-byte block had the wrong size.
    InvalidBlockLength,
    /// A stick range was malformed.
    InvalidStickRange,
    /// An IMU bias was not finite.
    InvalidImuBias,
}

/// Parses SDL's documented factory and optional user calibration blocks.
///
/// The supplied blocks are not retained after the returned typed calibration is
/// built. The serial-number block is intentionally not part of this API.
///
/// # Errors
///
/// Rejects malformed block lengths, ranges, and floating-point bias values.
pub fn parse_calibration(
    gyro_block: &[u8],
    left_stick_block: &[u8],
    right_stick_block: &[u8],
    acceleration_block: &[u8],
    trigger_block: &[u8],
    left_user_block: &[u8],
    right_user_block: &[u8],
) -> Result<(Bee021Calibration, CalibrationStatus), CalibrationError> {
    for block in [
        gyro_block,
        left_stick_block,
        right_stick_block,
        acceleration_block,
        trigger_block,
        left_user_block,
        right_user_block,
    ] {
        if block.len() != 64 {
            return Err(CalibrationError::InvalidBlockLength);
        }
    }
    let gyro_bias = [
        read_f32(gyro_block, 4)?,
        read_f32(gyro_block, 8)?,
        read_f32(gyro_block, 12)?,
    ];
    let acceleration_bias = [
        read_f32(acceleration_block, 12)?,
        read_f32(acceleration_block, 16)?,
        read_f32(acceleration_block, 20)?,
    ];
    let mut left_stick = parse_stick(&left_stick_block[0x28..])?;
    let mut right_stick = parse_stick(&right_stick_block[0x28..])?;
    let left_user_override = if let Some(stick) = user_stick(left_user_block)? {
        left_stick = stick;
        true
    } else {
        false
    };
    let right_user_override = if let Some(stick) = user_stick(right_user_block)? {
        right_stick = stick;
        true
    } else {
        false
    };
    Ok((
        Bee021Calibration {
            left_stick,
            right_stick,
            left_trigger_zero: trigger_block[0],
            right_trigger_zero: trigger_block[1],
            gyro_bias,
            acceleration_bias,
        },
        CalibrationStatus {
            factory_valid: true,
            left_user_override,
            right_user_override,
        },
    ))
}

fn user_stick(block: &[u8]) -> Result<Option<StickCalibration>, CalibrationError> {
    if block[0] == 0xb2 && block[1] == 0xa1 {
        parse_stick(&block[2..]).map(Some)
    } else {
        Ok(None)
    }
}

fn parse_stick(data: &[u8]) -> Result<StickCalibration, CalibrationError> {
    let x = AxisCalibration {
        neutral: unpack_low(data[0], data[1]),
        positive_span: unpack_low(data[3], data[4]),
        negative_span: unpack_low(data[6], data[7]),
    };
    let y = AxisCalibration {
        neutral: unpack_high(data[1], data[2]),
        positive_span: unpack_high(data[4], data[5]),
        negative_span: unpack_high(data[7], data[8]),
    };
    if !valid_axis(x) || !valid_axis(y) {
        return Err(CalibrationError::InvalidStickRange);
    }
    Ok(StickCalibration { x, y })
}

const fn valid_axis(axis: AxisCalibration) -> bool {
    axis.neutral != 0
        && axis.neutral <= 4_095
        && axis.positive_span != 0
        && axis.positive_span <= 4_095
        && axis.negative_span != 0
        && axis.negative_span <= 4_095
}

fn read_f32(data: &[u8], offset: usize) -> Result<f32, CalibrationError> {
    let value = f32::from_le_bytes(data[offset..offset + 4].try_into().expect("fixed bounds"));
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CalibrationError::InvalidImuBias)
    }
}

const fn unpack_low(low: u8, packed: u8) -> u16 {
    low as u16 | (((packed & 0x0f) as u16) << 8)
}

const fn unpack_high(packed: u8, high: u8) -> u16 {
    (packed >> 4) as u16 | ((high as u16) << 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stick_block(offset: usize) -> [u8; 64] {
        let mut block = [0_u8; 64];
        block[offset..offset + 9]
            .copy_from_slice(&[0x00, 0x08, 0x80, 0xff, 0x0f, 0xff, 0xff, 0x0f, 0xff]);
        block
    }

    #[test]
    fn parses_factory_calibration_without_exposing_blocks() {
        let mut gyro = [0_u8; 64];
        gyro[4..8].copy_from_slice(&1.0_f32.to_le_bytes());
        gyro[8..12].copy_from_slice(&2.0_f32.to_le_bytes());
        gyro[12..16].copy_from_slice(&3.0_f32.to_le_bytes());
        let left = stick_block(0x28);
        let right = stick_block(0x28);
        let mut acceleration = [0_u8; 64];
        acceleration[12..16].copy_from_slice(&4.0_f32.to_le_bytes());
        acceleration[16..20].copy_from_slice(&5.0_f32.to_le_bytes());
        acceleration[20..24].copy_from_slice(&6.0_f32.to_le_bytes());
        let mut triggers = [0_u8; 64];
        triggers[0] = 12;
        triggers[1] = 13;
        let (calibration, status) = parse_calibration(
            &gyro,
            &left,
            &right,
            &acceleration,
            &triggers,
            &[0; 64],
            &[0; 64],
        )
        .expect("valid calibration");
        assert!(status.factory_valid);
        assert_eq!(calibration.left_trigger_zero, 12);
        assert_eq!(calibration.right_trigger_zero, 13);
        assert_eq!(calibration.left_stick.x.normalize(0x800), 0);
    }

    #[test]
    fn rejects_invalid_ranges_and_nonfinite_biases() {
        let malformed = [0_u8; 64];
        assert!(matches!(
            parse_calibration(
                &malformed, &malformed, &malformed, &malformed, &malformed, &malformed, &malformed,
            ),
            Err(CalibrationError::InvalidStickRange)
        ));
    }
}
