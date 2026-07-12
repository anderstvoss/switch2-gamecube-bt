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

Do not use WSL2. Prioritize an ESP32-S3 BLE bridge as the first working-
connection path because it runs a separate BLE stack from the Windows host.
First create a project-owned, read-only serial bridge diagnostic that can prove
bridge availability and report only sanitized discovery metadata. Do not adopt
the public application's pairing, initialization, virtual-device, or output
logic.

## Consequences

- A compatible ESP32-S3 development board is not attached to the current host.
  Obtaining one, flashing its firmware, or attaching it requires user action
  and approval.
- Do not connect a controller through the bridge until the project-owned
  read-only diagnostic observes a sanitized candidate and the user authorizes
  the connection.
- The public bridge implementation is an interoperability lead, not a trusted
  production dependency. In particular, it includes pairing, initialization,
  virtual-device, and output behavior outside this project's current scope.
- The portable Rust contracts remain the integration boundary for both Linux
  and bridge transports.
