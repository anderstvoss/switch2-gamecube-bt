# Decision 0001: gate Switch 2 USB initialization

- Date: 2026-07-11
- Status: accepted

## Context

Passive Windows HID reads produced no BEE-021 input reports. SDL3 obtains wired
input through libusb bulk endpoints only after calibration reads and several
initialization commands, including commands whose purpose is unknown upstream.
Recent Steam clients provide a second observed wired implementation through
Steam Input, but their physical-device transport is not public. SDL history
contains explicit Steam interoperability changes, so Steam can validate the
final normalized behavior but cannot independently document packet semantics.

## Decision

Do not send the SDL3 sequence or any experimental controller command in the
passive-observation change. Before implementing USB initialization:

1. classify every command as read-only, volatile session state, or potentially
   persistent;
2. exclude firmware, bootloader, flash write, calibration write, reset, and
   pairing-material operations by construction;
3. document the expected reply shape and bound every transfer;
4. place the sequence behind an explicit BEE-021 USB identity check and user
   confirmation;
5. add a transport mock that proves ordering, timeouts, and rejection of unknown
   commands; and
6. request approval immediately before the first live initialization attempt.

After input is available, compare control presence, axis behavior, motion order,
and disconnect cleanup with Steam Input. Do not capture Steam process traffic,
inject into Steam, or infer proprietary implementation details from binaries.

Factory calibration may be read only after its address range and command
semantics are reviewed. Calibration contents must remain local and must not be
committed as hardware evidence.

## Consequences

USB input remains `not ready` rather than being labeled unsupported. Decoder and
Bluetooth work cannot rely on passive HID reads. The next implementation slice
is a test-only initialization protocol model and safety review, not live device
output.
