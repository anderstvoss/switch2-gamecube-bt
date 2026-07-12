# Windows USB described-command probe: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: four described non-rumble initialization commands
- State reached: commands acknowledged, input not ready
- Evidence confidence: observed once

## Sanitized result

On a freshly reconnected session, the project sent the four reviewed commands
in order: set feature-output mask, enable feature-output channels, select report
format `0x05`, and start streaming. Their reply lengths were respectively 12,
12, 8, and 12 bytes. No input reports arrived during the bounded ten-second
observation. The interface was released without another command.

No unknown, rumble, grip, flash, firmware, reset, pairing, calibration-write,
or cleanup command was sent. No raw command reply, input report, device path,
serial number, or machine identifier was retained.

## Interpretation

The transport and all four described commands are accepted, but that subset is
insufficient to activate the wired input stream. The remaining current SDL
sequence contains unknown-purpose, rumble-related, and charging-grip commands.
The project will not automatically copy those commands into its allowlist.

The next baseline uses the official current SDL3 wired driver as a maintained
reference implementation. This is not an attempt to replace SDL3's wired
support. The project-specific USB path exists to obtain controlled protocol
evidence and fixtures for the Bluetooth decoder and future Windows service.
