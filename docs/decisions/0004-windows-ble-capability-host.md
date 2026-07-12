# Windows BLE capability host

## Status

Accepted and implemented for local lab use. The user authorized creation and
installation of the test package and its local test certificate. Removal of
either artifact, or any broader Windows policy change, requires new approval.

## Context

The BEE-021 wireless path has been reclassified as a likely proprietary BLE
GATT protocol. The host's default adapter reports BLE and central-role support,
but the unpackaged Rust diagnostic receives no BLE advertisements. Microsoft's
BLE advertisement documentation requires the `bluetooth` package capability.

## Decision

Keep BLE discovery, decoding, and GATT protocol logic in the portable,
safe-Rust library. Add a minimal Windows MSIX capability host later, solely to
declare the Bluetooth device capability and launch the existing diagnostic or
service. The package host must not contain controller protocol logic, raw-log
persistence, pairing-key handling, or virtual-HID driver code.

## Consequences

- A package-backed BLE scan is required before interpreting an empty Windows
  advertisement watcher as a controller discovery failure.
- Creating a test certificate, installing the MSIX package, enabling developer
  features, or removing the package requires explicit user approval.
- The Linux BLE backend and future controller models continue to use the same
  portable discovery and transport contracts.
