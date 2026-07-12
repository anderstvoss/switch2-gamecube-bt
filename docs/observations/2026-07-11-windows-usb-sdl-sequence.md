# Windows USB exact SDL sequence: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: exact pinned SDL initialization and HID readiness check
- State reached: HID input ready
- Evidence confidence: observed once

## Sanitized result

On a freshly reconnected session, the project sent the exact ten-packet
initialization sequence from pinned SDL revision `82141a2`. Ordered command
reply lengths were 9, 12, 12, 8, 12, 8, 8, 8, 8, and 12 bytes.

No state reports appeared on bulk endpoint `0x82`. Source review then confirmed
that SDL uses bulk IN only for initialization replies; its update loop reads
controller state through the HID interface. A bounded HID observation in the
same initialized session immediately received 64 reports with ID `0x05`, each
64 bytes long.

No raw command reply or input report was retained. No firmware, flash write,
calibration write, reset, pairing, or undocumented persistent-storage command
was sent.

## Corrected transport model

USB interface 1 (`WinUSB`) carries initialization commands and their replies.
USB interface 0 (`HidUsb`) carries continuous 64-byte state reports after
initialization. The project's prior probes correctly measured an empty
post-command bulk stream but incorrectly treated that as absence of controller
state. Their recorded command replies remain valid; their `input not ready`
interpretation is superseded by this observation for the exact SDL sequence.

## Consequence

The wired baseline is ready for sanitized decoding and physical input
verification. SDL remains the source of truth for the established wired report
layout. Unknown, rumble, and grip packets from its complete initialization
remain isolated from the normal project allowlist pending further analysis.
