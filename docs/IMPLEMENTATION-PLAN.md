# Switch 2 GameCube Bluetooth Implementation Plan

This roadmap builds native Windows 11 support for the official Switch 2
GameCube controller (BEE-021) first. Protocol, decoding, and application
contracts remain platform-neutral so Linux backends and other Switch 2
controllers can be added without changing the Windows-facing workflow.

## Working rules

- Keep production Rust compatible with `unsafe_code = "forbid"`.
- Never write controller firmware, bootloader state, calibration, pairing
  secrets, or undocumented persistent controller storage.
- Keep Bluetooth and USB hardware tests opt-in and user supervised.
- Store only sanitized evidence; never commit addresses, serial numbers, raw
  captures, logs, generated drivers, credentials, or local machine paths.
- Land small commits only after the relevant deterministic checks pass.
- Pause for user confirmation at every hardware checkpoint below.

## Ordered goals

### Goal 0: Windows lab and evidence workflow

Document the native Windows lab, preserve the WSL2 comparison lab, and track
support claims by connection type, reached state, and evidence confidence.

Exit gate: another contributor can record a sanitized Windows observation
without changing application code.

### Goal 1: Portable domain and protocol contracts

Implement opaque identities, capabilities, validated connection states,
deadlines, cancellation, privacy-safe errors, versioned events, raw reports,
normalized input, output requests, and a controller registry. Cover the
contracts with deterministic tests.

Exit gate: all domain behavior is testable without an operating system or
controller.

### Goal 2: Fake backend and diagnostic CLI

Add a deterministic backend and CLI commands for adapter inventory, scanning,
pairing, connection, information, bounded observation, input testing, and
sanitized diagnostics. Support human and versioned JSON output.

Exit gate: the complete workflow, including cancellation and failures, runs
against the fake backend.

### Goal 3: Read-only Windows USB baseline

After requesting that the user connect the controller by USB-C, enumerate its
HID interfaces and compare sanitized observations with SDL3. Build the first
BEE-021 decoder exclusively from repeatable evidence.

Exit gate: every verified wired input has a sanitized fixture and deterministic
decoder test. Unknown and output reports remain disabled.

Current evidence: Windows exposes BEE-021 HID metadata and its report
descriptor, but passive HID input remains silent. Before live report capture,
model and review the libusb bulk initialization used by SDL3, classify every
operation by persistence risk, reject unknown commands, and request approval
before the first live initialization attempt. Use recent Steam Input as a
black-box wired behavior reference while treating SDL3's public libusb driver as
the auditable implementation reference. See Decision 0001.

The test-only initialization model now preflights the complete audited order.
Four understood packets remain `candidate volatile`, and six unknown,
rumble-related, or non-BEE-021 steps block all execution. No live transport or
public executable-plan constructor exists. See Decision 0002.

Windows' existing `WinUSB` binding on USB interface 1 can now be inspected
without replacing a driver or claiming the interface. A live read-only
descriptor check found one 64-byte bulk IN endpoint at `0x82` and one 64-byte
bulk OUT endpoint at `0x02`. Endpoint selection remains dynamic, matching the
auditable SDL3 approach. The next checkpoint is to add a bounded transport
implementation; it must remain unreachable from the CLI until a separately
reviewed initialization slice is approved for live use.

### Goal 4: Native Windows Bluetooth

Implement Windows adapter and device inventory before pairing. After requesting
SYNC mode and confirming the selected device, add cancellable pairing,
connection, and independent HID-readiness checks. Do not access or persist link
keys.

Exit gate: Windows reports discovery, pairing, connection, and HID readiness as
separate evidence-backed states.

### Goal 5: Bluetooth decoding and safe outputs

Compare Bluetooth reports with the USB baseline. Add only verified session
initialization and decoding. Test each physical control, motion, battery,
sleep/wake, and reconnect separately. Request user approval before the first
evidence-backed volatile output command.

Exit gate: every supported feature is backed by live evidence and fixtures;
unobserved features remain explicitly unverified.

### Goal 6: Windows virtual gamepad

Define a versioned service-to-driver contract and a generic gamepad mapping.
Build a minimal KMDF/VHF source driver outside the safe Rust workspace, keeping
all Bluetooth and controller logic in Rust. Request permission before enabling
test-signing or installing the driver.

Exit gate: Windows games see a virtual controller, and disconnects or service
failure always neutralize input and remove stale device state.

### Goal 7: Automatic Windows service

Add automatic reconnect and a small setup/diagnostic client. Validate clean
installation, startup, shutdown, sleep/wake, Bluetooth disablement, driver
removal, and rollback.

Exit gate: normal play needs no foreground diagnostic process.

### Goal 8: Portability and more controllers

Implement Linux BlueZ/hidraw against the shared contracts and retain SDL3 as a
validation or consumer adapter. Add other Switch 2 controllers only through
separate evidence-backed registry implementations.

Exit gate: platform and controller additions do not change the domain,
diagnostic, or mapping contracts.

### Goal 9: Release hardening

Run pre-commit and pre-push gates, dependency review, parser and IOCTL fuzzing,
Windows driver analysis, and clean-machine installation tests. Document
test-signing and production-signing requirements.

Exit gate: all claimed support is present in the support matrix and every local
release gate passes.

## Hardware checkpoints

Stop and request user action before:

1. connecting the controller over USB-C;
2. exercising buttons, sticks, triggers, or motion controls;
3. unplugging USB and entering Bluetooth SYNC mode;
4. confirming a discovered device for pairing;
5. sending the first verified volatile output command;
6. enabling Windows test-signing or installing/removing the VHF driver; and
7. re-pairing with a Switch 2 when console hardware becomes available.

USB is required for the initial protocol baseline but is not a normal Bluetooth
runtime dependency.

## Commit and verification policy

Each goal is divided into the smallest reviewable vertical slices. Before each
commit, run formatting, checking, Clippy, tests, and the repository pre-commit
hooks. Before release or publication, also run the pre-push dependency,
advisory, secret, and artifact checks. Update this plan after hardware evidence
changes an assumption or exposes a new protocol boundary.
