# Windows USB inventory observation: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: read-only HID inventory
- State reached: discovered
- Evidence confidence: observed once

## Sanitized result

The operating system identified a healthy Nintendo GameCube Controller with USB
vendor ID `057e` and product ID `2073`. The read-only Rust inventory found one
USB HID interface with Generic Desktop usage page `0001`, Game Pad usage `0005`,
and interface number `0`.

No device handle was opened. No report descriptor, input report, serial number,
device path, output report, feature report, pairing material, calibration, or
firmware data was read or written.
