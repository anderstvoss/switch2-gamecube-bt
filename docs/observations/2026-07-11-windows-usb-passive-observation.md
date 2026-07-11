# Windows USB passive observation: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: report-descriptor fingerprint and passive HID input observation
- State reached: discovered, HID input not ready
- Evidence confidence: observed once

## Sanitized result

The HID report descriptor was 119 bytes with SHA-256 digest
`6c85dffb6cc84ba82498763a4a3ae2a4f2b31389e691d779aa4c94b0ee0c8b22`.
The controller accepted a read-only HID handle, but returned no input reports
during a 10-second observation while every button, both sticks, both triggers,
the D-pad, and motion were exercised.

No report bytes, device path, serial number, output report, feature report,
calibration content, pairing material, or firmware data was exported or
written.

## Upstream comparison

SDL source at commit
[`82141a2`](https://github.com/libsdl-org/SDL/blob/82141a2439acc111661102ba6f968c85e71cff40/src/joystick/hidapi/SDL_hidapi_switch2.c)
uses libusb bulk endpoints for Switch 2 controllers over USB. Its implementation
claims the USB interface, reads factory calibration, and sends a multi-step
initialization sequence before processing state packets. Some initialization
steps are explicitly described as having unknown purpose. The same source marks
Switch 2 Bluetooth support as not yet implemented.

This comparison explains the passive-read result but does not authorize copying
or sending the upstream initialization sequence.

## Steam Input comparison

The controller owner reports that recent Steam clients and Steam Input accept
this BEE-021 controller in wired mode. Valve documents Steam Input as the layer
that receives physical-device input and translates it for games in native or
legacy modes. Recent Steam client beta notes also reference fixes for Switch 2
controller regressions.

SDL's public Switch 2 history provides additional interoperability evidence:

- commit [`a798da2e`](https://github.com/libsdl-org/SDL/commit/a798da2e) added the
  NSO GameCube controller through libusb; and
- commit [`55a566a6`](https://github.com/libsdl-org/SDL/commit/55a566a6) states
  that Steam expects gyroscope data before accelerometer data.

Steam's physical-device transport implementation is proprietary. These facts
show that Steam Input is a useful end-to-end wired reference and that SDL's
Switch 2 work accounts for Steam interoperability, but they do not prove that
Steam sends the same initialization packets as SDL. The project must therefore
validate its own allowlisted transport behavior rather than treating Steam as
source-level protocol documentation.
