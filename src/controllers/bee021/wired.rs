//! Evidence-backed BEE-021 wired report decoding.

use crate::protocol::{Axis, Button, InputFrame, MotionSample};

const STANDARD_GRAVITY: f32 = 9.806_65;
const ACCELERATION_SCALE: f32 = STANDARD_GRAVITY * 8.0 / i16::MAX as f32;
const PROVISIONAL_GYRO_SCALE: f32 = 34.8 / i16::MAX as f32;

/// Wired state report identifier selected by SDL's initialization sequence.
pub const WIRED_REPORT_ID: u8 = 0x05;
/// Complete wired state report length.
pub const WIRED_REPORT_LENGTH: usize = 64;

/// Bounded wired report decoding failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WiredDecodeError {
    /// The report identifier is not the verified wired state format.
    UnexpectedReportId(u8),
    /// The report is not exactly the verified 64-byte layout.
    InvalidLength(usize),
}

/// Decodes one SDL-format BEE-021 wired state report.
///
/// Stick and trigger values use descriptor-wide fallback normalization until
/// read-only per-device calibration is implemented. Button offsets and packed
/// 12-bit stick fields follow pinned SDL source.
///
/// # Errors
///
/// Rejects reports with an unexpected identifier or length before indexing.
pub fn decode_wired_report(report: &[u8]) -> Result<InputFrame, WiredDecodeError> {
    if report.len() != WIRED_REPORT_LENGTH {
        return Err(WiredDecodeError::InvalidLength(report.len()));
    }
    if report[0] != WIRED_REPORT_ID {
        return Err(WiredDecodeError::UnexpectedReportId(report[0]));
    }

    let mut frame = InputFrame::default();
    add_button(&mut frame, report[5], 0x01, Button::X);
    add_button(&mut frame, report[5], 0x02, Button::Y);
    add_button(&mut frame, report[5], 0x04, Button::A);
    add_button(&mut frame, report[5], 0x08, Button::B);
    add_button(&mut frame, report[5], 0x40, Button::R);
    add_button(&mut frame, report[5], 0x80, Button::Z);

    add_button(&mut frame, report[6], 0x02, Button::Start);
    add_button(&mut frame, report[6], 0x10, Button::Home);
    add_button(&mut frame, report[6], 0x20, Button::Capture);
    add_button(&mut frame, report[6], 0x40, Button::Chat);

    add_button(&mut frame, report[7], 0x01, Button::DpadDown);
    add_button(&mut frame, report[7], 0x02, Button::DpadUp);
    add_button(&mut frame, report[7], 0x04, Button::DpadRight);
    add_button(&mut frame, report[7], 0x08, Button::DpadLeft);
    add_button(&mut frame, report[7], 0x40, Button::L);
    add_button(&mut frame, report[7], 0x80, Button::ZL);

    frame.axes.insert(
        Axis::LeftX,
        normalize_12bit(unpack_low(report[11], report[12])),
    );
    frame.axes.insert(
        Axis::LeftY,
        -normalize_12bit(unpack_high(report[12], report[13])),
    );
    frame.axes.insert(
        Axis::CStickX,
        normalize_12bit(unpack_low(report[14], report[15])),
    );
    frame.axes.insert(
        Axis::CStickY,
        -normalize_12bit(unpack_high(report[15], report[16])),
    );
    frame
        .axes
        .insert(Axis::LeftTrigger, normalize_8bit(report[61]));
    frame
        .axes
        .insert(Axis::RightTrigger, normalize_8bit(report[62]));
    let sensor_timestamp = u32::from_le_bytes([report[43], report[44], report[45], report[46]]);
    if sensor_timestamp != 0 {
        frame.motion.push(MotionSample {
            acceleration: [
                f32::from(read_i16(report, 49)) * ACCELERATION_SCALE,
                f32::from(read_i16(report, 53)) * ACCELERATION_SCALE,
                -f32::from(read_i16(report, 51)) * ACCELERATION_SCALE,
            ],
            angular_velocity: [
                f32::from(read_i16(report, 55)) * PROVISIONAL_GYRO_SCALE,
                f32::from(read_i16(report, 59)) * PROVISIONAL_GYRO_SCALE,
                -f32::from(read_i16(report, 57)) * PROVISIONAL_GYRO_SCALE,
            ],
        });
    }
    Ok(frame)
}

fn add_button(frame: &mut InputFrame, byte: u8, mask: u8, button: Button) {
    if byte & mask != 0 {
        frame.buttons.insert(button);
    }
}

const fn unpack_low(low: u8, packed: u8) -> u16 {
    low as u16 | (((packed & 0x0f) as u16) << 8)
}

const fn unpack_high(packed: u8, high: u8) -> u16 {
    (packed >> 4) as u16 | ((high as u16) << 4)
}

fn normalize_12bit(value: u16) -> i16 {
    i16::try_from((((i32::from(value) * 65_535) / 4_095) - 32_768).clamp(-32_767, 32_767))
        .expect("normalized value is clamped to i16")
}

fn normalize_8bit(value: u8) -> i16 {
    i16::try_from(((i32::from(value) * 65_535 / 255) - 32_768).clamp(-32_767, 32_767))
        .expect("normalized value is clamped to i16")
}

fn read_i16(report: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([report[offset], report[offset + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_report_shapes() {
        assert_eq!(
            decode_wired_report(&[]),
            Err(WiredDecodeError::InvalidLength(0))
        );
        let mut report = [0_u8; WIRED_REPORT_LENGTH];
        report[0] = 0x04;
        assert_eq!(
            decode_wired_report(&report),
            Err(WiredDecodeError::UnexpectedReportId(0x04))
        );
    }

    #[test]
    fn decodes_verified_buttons_and_packed_axes() {
        let mut report = [0_u8; WIRED_REPORT_LENGTH];
        report[0] = WIRED_REPORT_ID;
        report[5] = 0x45;
        report[6] = 0x72;
        report[7] = 0xc6;
        report[11] = 0xff;
        report[12] = 0x0f;
        report[61] = 0xff;
        let frame = decode_wired_report(&report).expect("valid report");
        for button in [
            Button::X,
            Button::A,
            Button::R,
            Button::Start,
            Button::Home,
            Button::Capture,
            Button::Chat,
            Button::DpadUp,
            Button::DpadRight,
            Button::L,
            Button::ZL,
        ] {
            assert!(frame.buttons.contains(&button));
        }
        assert_eq!(frame.axes[&Axis::LeftX], 32_767);
        assert_eq!(frame.axes[&Axis::LeftTrigger], 32_767);
    }

    #[test]
    fn decodes_motion_in_sdl_axis_order() {
        let mut report = [0_u8; WIRED_REPORT_LENGTH];
        report[0] = WIRED_REPORT_ID;
        report[43] = 1;
        report[49..51].copy_from_slice(&1_000_i16.to_le_bytes());
        report[51..53].copy_from_slice(&(-2_000_i16).to_le_bytes());
        report[53..55].copy_from_slice(&3_000_i16.to_le_bytes());
        report[55..57].copy_from_slice(&4_000_i16.to_le_bytes());
        report[57..59].copy_from_slice(&(-5_000_i16).to_le_bytes());
        report[59..61].copy_from_slice(&6_000_i16.to_le_bytes());
        let frame = decode_wired_report(&report).expect("valid motion report");
        let motion = frame.motion[0];
        assert!(motion.acceleration[0] > 0.0);
        assert!(motion.acceleration[1] > motion.acceleration[0]);
        assert!(motion.acceleration[2] > 0.0);
        assert!(motion.angular_velocity[0] > 0.0);
        assert!(motion.angular_velocity[1] > motion.angular_velocity[0]);
        assert!(motion.angular_velocity[2] > 0.0);
    }
}
