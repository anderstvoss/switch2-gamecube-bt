# Windows USB start-stream probe: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: approved one-packet start-stream probe
- State reached: command acknowledged, input not ready
- Evidence confidence: observed once

## Sanitized result

After revalidating USB identity `057e:2073` and the interface 1 bulk endpoint
layout, the project claimed interface 1 and sent the reviewed 16-byte
start-stream packet once. The controller returned a 12-byte command reply. No
64-byte input reports arrived during the following bounded ten-second window.
The interface was released without sending a cleanup or follow-on command.

Only reply/report lengths and counts crossed the transport boundary. No reply
bytes, input bytes, serial number, device path, calibration content, pairing
material, or machine-specific identifier was retained.

## Interpretation

The successful reply demonstrates that the Windows WinUSB transport, endpoint
selection, exact write, and bounded read path are functioning. It also shows
that the start-stream packet alone is insufficient to produce the current
64-byte input stream in this session.

The result does not prove whether report-format selection alone is the missing
prerequisite, whether feature-output setup is required, or whether the command
has persistent effects. It does not authorize unknown, rumble, grip, flash,
firmware, reset, pairing, or calibration-write commands.

## Next experiment

Reconnect USB to establish a fresh session. Then send the evidence-backed
report-format `0x05` command followed by start-stream, with one bounded reply
per command and the same ten-second input observation. Stop on either command
failure or if no reports appear; do not automatically add other SDL packets.
