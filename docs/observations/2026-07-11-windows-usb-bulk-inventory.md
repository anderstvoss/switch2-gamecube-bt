# Windows USB bulk inventory: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: active configuration and bulk endpoint inspection
- State reached: discovered, bulk transport shape verified
- Evidence confidence: observed once

## Sanitized result

The controller's existing `WinUSB` binding exposes interface 1 with one bulk IN
endpoint at `0x82` and one bulk OUT endpoint at `0x02`. Both endpoints report a
64-byte maximum packet size. The project discovers these values from the active
configuration descriptor rather than hard-coding them.

The observation opened the device-level handle needed to read the descriptor.
It did not claim interface 1 and did not perform a bulk, control, feature, or
output transfer. No serial number, device path, raw descriptor, report content,
pairing material, or machine-specific identifier was recorded.

## Interpretation

The endpoint shape matches SDL's dynamic search for a bulk endpoint in each
direction. This verifies that the installed Windows device configuration can
support a later WinUSB transport; it does not verify input readiness or command
safety.
