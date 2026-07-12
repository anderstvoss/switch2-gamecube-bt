# Windows USB report-format probe: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: approved report-format `0x05` plus start-stream probe
- State reached: commands acknowledged, input not ready
- Evidence confidence: observed once

## Sanitized result

On a freshly reconnected session, the project revalidated USB identity
`057e:2073` and the interface 1 bulk endpoints. It sent report-format `0x05`
once and received an 8-byte reply, then sent start-stream once and received a
12-byte reply. No 64-byte input reports arrived during the bounded ten-second
observation. The interface was released without another command.

Only ordered reply lengths and report counts crossed the transport boundary.
No reply bytes, input bytes, serial number, device path, calibration content,
pairing material, or machine-specific identifier was retained.

## Interpretation

The result shows both input-oriented commands are accepted but insufficient to
enable input together. It narrows the next evidence-backed probe to the four
described, non-rumble commands: set feature-output mask, enable those feature
outputs, select report format `0x05`, and start streaming.

The observation does not establish persistence behavior and does not authorize
unknown, rumble, grip, flash, firmware, reset, pairing, or calibration-write
commands.
