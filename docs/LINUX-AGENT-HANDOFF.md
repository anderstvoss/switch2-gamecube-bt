# Native Linux BLE Lab Handoff

This branch is a dedicated handoff for agents working on a **native Linux
machine** with physical Bluetooth hardware. It is not a WSL1 or WSL2 workflow.
Use this document with `AGENTS.md`, `docs/PROJECT-MEMORY.md`, and
`docs/IMPLEMENTATION-PLAN.md` before changing code or touching controller
hardware.

## Objective

Establish the first evidence-backed Bluetooth Low Energy path to the official
Nintendo Switch 2 GameCube controller, BEE-021. The immediate target is only:

```text
adapter available -> sanitized candidate discovered
```

Do not treat discovery as pairing, connection, GATT readiness, input readiness,
or controller support.

## What is already known

- BEE-021 is verified over USB as Nintendo `057e:2073`. Its wired input,
  motion, and read-only calibration baseline is implemented and documented.
- Two Windows BLE discovery mechanisms were verified under package identity:
  the advertisement watcher and unpaired BLE device-selector watcher. Each
  observed zero candidates during a supervised eight-second SYNC attempt.
- This is negative evidence for those Windows mechanisms only. It does not
  prove a controller fault, rule out BLE, or rule out direct Linux discovery.
- Public Switch 2 work identifies vendor service UUID
  `ab7de9be-89fe-49ad-828f-118f09df7fd0` as a useful lead. The audited BlueZ
  implementation states that it was developed and tested with Pro Controller 2,
  not BEE-021. Do not reuse its initialization sequence as BEE-021 fact.
- A public Windows implementation can use either the normal Windows BLE stack
  or an ESP32-S3 bridge. Its pairing, initialization, output, virtual-device,
  and persistence logic are out of scope for this lab.

## Required safety boundaries

- Keep `unsafe_code = "forbid"` in the Rust workspace.
- Never write firmware, calibration, pairing/link keys, flash, or undocumented
  persistent controller storage.
- Do not retain Bluetooth addresses, controller serials, raw packets, raw
  advertisements, BlueZ databases, HCI captures, host paths, usernames, or
  logs in tracked files.
- Do not pair, trust, bond, connect, access GATT, subscribe to notifications,
  or send controller commands until the previous state has live evidence and
  the user explicitly approves the next physical checkpoint.
- Do not use an ESP32 bridge, virtual gamepad, or output command in this phase.

## Native Linux prerequisites

- A native Linux installation with a supported Bluetooth adapter. Prefer a
  dedicated USB BLE adapter so other host radios and pairings are unaffected.
- BlueZ and its normal management tools installed through the distribution's
  package manager.
- A clean repository checkout of this branch and the local Rust toolchain.
- The controller nearby, but do not request SYNC until the diagnostic is built
  and ready to start.

Record only adapter capability labels and sanitized outcomes. Do not commit
machine-specific package output or BlueZ service logs.

## Ordered work plan

### 1. Read-only Linux adapter status

Implement or validate a narrowly scoped Linux adapter-status diagnostic behind
the portable platform boundary. It should report only whether a BlueZ adapter
is present, powered, and supports LE scanning. Cover parsing and error mapping
with deterministic tests; do not require hardware in CI.

### 2. Bounded sanitized discovery

Add a cancellable, one-to-ten-second BlueZ LE discovery command. It must
deduplicate candidates, expose only a short digest of the address plus
evidence-backed service-match flags, and discard raw advertisement data
immediately. Keep every discovery event bounded and redacted.

Do not add a name, model, or service filter until its source is documented.
The first scan should be broad enough to determine whether Linux sees any BLE
advertisements during BEE-021 SYNC.

### 3. Physical discovery checkpoint

After the command is prebuilt and deterministic checks pass, ask the user for
`ready for native Linux BLE SYNC`. Start the bounded scan first, then tell the
user to hold SYNC immediately. Record a sanitized observation and update the
support matrix.

If no candidate appears, stop hardware experimentation and inspect BlueZ
adapter state, scan mode, and alternate adapters before asking for another
controller action.

### 4. Candidate confirmation gate

If a candidate appears, display only its short digest and evidence-backed
service match. Stop and ask the user to confirm it before opening a connection.
Do not pair or send a command.

### 5. Future connection gate

Only after explicit approval, add a bounded **read-only** GATT service-
discovery client. It must make no pairing, bonding, writes, descriptor writes,
notification subscriptions, or persistent changes. Service discovery evidence
is required before any BEE-021 session command is considered.

## Public research leads

- Microsoft Windows BLE docs remain useful for contrast, but are not a Linux
  transport authority.
- BlueZ Switch 2 plugin patch: protocol and characteristic-role lead only;
  coverage is Pro Controller 2-specific.
  <https://www.spinics.net/lists/linux-bluetooth/msg127380.html>
- Public Switch 2 input viewer: handle and feature-flag lead only. It includes
  writes and sensitive reads prohibited by this project.
  <https://gist.github.com/ndeadly/7d27aa63e2f653a902a2474dbcbc08b3>
- Windows/ESP32 bridge reference: interoperability lead only, not a dependency
  or safe implementation source.
  <https://github.com/TommyWabg/switch2-controllers-windows10-gyro>

## Validation and handoff discipline

Before each commit run formatting, `cargo check`, Clippy with warnings denied,
tests, and `pre-commit run --all-files`. Run the pre-push checks before any
publication or release step. Keep commits narrow and update the memory, plan,
support matrix, and a sanitized observation whenever hardware evidence changes
the roadmap.

Do not open a PR from this handoff branch until the native Linux discovery
milestone has a reviewed, evidence-backed result.
