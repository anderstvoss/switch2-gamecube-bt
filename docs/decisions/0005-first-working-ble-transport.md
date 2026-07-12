# First working BLE transport

## Status

Accepted for planning. No external hardware, firmware flashing, pairing, or
connection is authorized by this decision.

## Context

Two verified, package-identity Windows discovery mechanisms returned no BEE-021
candidate during supervised SYNC attempts. A public Windows-oriented Switch 2
implementation was audited as a transport lead. It uses a separate ESP32-S3
BLE bridge rather than the Windows Bluetooth stack.

The current development host has no WSL distribution installed and no USB
adapter-passthrough tool available, so a native Linux/BlueZ comparison cannot
run yet.

## Decision

Prioritize a direct Linux BlueZ experiment as the first working-connection
path: install WSL2 with a supported Linux distribution and attach a dedicated
USB Bluetooth adapter to that environment. Use only bounded, read-only BlueZ
discovery first. Keep the ESP32-S3 bridge as an alternative if direct BlueZ
discovery cannot see BEE-021.

## Consequences

- Installing WSL2, installing a Linux distribution, installing adapter
  passthrough tooling, or attaching an adapter changes local Windows state and
  requires explicit user approval.
- A dedicated USB Bluetooth adapter is preferred so the existing Windows radio
  is not disrupted.
- Do not flash, configure, or connect an ESP32-S3 bridge without explicit
  approval. Its public implementation is an interoperability lead, not a
  trusted production dependency.
- The portable Rust contracts remain the integration boundary for both Linux
  and bridge transports.
