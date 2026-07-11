# Switch 2 GameCube Bluetooth Development Specification

## 1. Purpose

`switch2-gamecube-bt` is a cross-platform tool for discovering, pairing,
connecting to, inspecting, and eventually exposing Nintendo Switch 2
controllers as usable GameCube-style input devices over Bluetooth.

The project begins with a Windows host and WSL2 development environment. The
first implementation target is a reliable command-line diagnostic and pairing
workflow. A graphical interface and input translation are built on top of the
same library after the Bluetooth and HID behavior is understood.

This document is the development contract. It records the intended behavior,
boundaries, unknowns, and completion criteria. Hardware observations must be
updated as evidence is collected; protocol assumptions are not requirements.

## 2. Current Status and Assumptions

The repository is a Rust workspace with lint, test, dependency, secret, and
artifact checks already established. The product
implementation has not started.

The current working assumptions are:

- Original Switch Pro and Joy-Con Bluetooth support is mature on several
  platforms, but Switch 2 controller support is less consistently documented.
- Windows support may be more reliable through the native Bluetooth and HID
  APIs than through WSL2.
- WSL2 is suitable for Linux-side development and protocol inspection, but
  direct access to a host Bluetooth adapter may require a USB adapter and
  explicit device passthrough.
- Linux is the best initial environment for low-level observation because
  BlueZ, `hidraw`, `evdev`, and `uinput` expose useful layers independently.
- A controller may pair successfully without producing usable input reports;
  the tool must distinguish those states.
- Switch 2-specific report formats, authentication, firmware behavior, and
  Joy-Con pairing semantics are open research questions.

## 3. Goals

### Initial release goals

1. Show Bluetooth adapters and nearby candidate controllers.
2. Guide the user through pairing, bonding/trusting, connecting, and
   disconnecting a controller.
3. Persist only the minimum safe metadata needed to reconnect a known device.
4. Report each stage and failure reason in plain language, with diagnostic
   detail available for developers.
5. Capture sanitized device metadata and HID report observations for protocol
   research.
6. Provide a stable library API that can be used by both a CLI and a future
   GUI.
7. Work natively on Windows, Linux, and macOS where the platform APIs permit
   the requested operation, with clear capability reporting where they do not.

### Later goals

- Decode buttons, axes, battery, connection state, and controller identity.
- Normalize controller input into a platform-neutral event model.
- Expose a virtual GameCube-compatible device or mapping profile.
- Support two Joy-Con units as one logical controller when the platform and
  protocol allow it.
- Add a GUI with guided pairing, live input testing, profiles, and exportable
  diagnostics.

### Explicit non-goals for the first release

- Firmware updates or controller modification.
- Emulating the Nintendo Switch console or bypassing authentication.
- Shipping proprietary controller firmware or extracted copyrighted assets.
- Promising support for a controller model before it is verified on real
  hardware.
- Running privileged services by default.

## 4. User Workflows

### Guided pairing

The user selects an adapter, puts a controller into its documented pairing
mode, and starts discovery. The tool presents candidate devices with a stable
display name, address identifier where the platform exposes one, class/type,
and signal or discovery status.

The user selects a device and chooses `Pair`. The tool then attempts the
platform-appropriate sequence: discover, pair, bond/trust if applicable,
connect, and verify that a HID endpoint is present. Each transition is
reported independently so a failure at input verification is not mislabeled
as a pairing failure.

### Reconnect

For a previously known device, the user chooses `Connect`. The tool checks
whether the adapter and device are available, reconnects, verifies HID
availability, and reports the final state. Stored metadata must never include
link keys, session keys, or other credentials.

### Diagnostics

The user can run a diagnostic session that records timestamps, platform
capabilities, adapter information, device identity fields exposed by the OS,
connection transitions, report lengths, and redacted report samples. Raw
captures are opt-in, clearly marked, and excluded from normal logs and source
control.

### CLI-first commands

The initial CLI should provide equivalent operations using stable exit codes:

```text
s2bt adapters
s2bt scan [--duration <seconds>]
s2bt pair <device>
s2bt trust <device>
s2bt connect <device>
s2bt disconnect <device>
s2bt info <device>
s2bt diagnose <device> [--output <directory>]
```

Command names and flags are provisional until the first CLI prototype is
usable. Every command must support a concise human-readable mode and a
machine-readable JSON mode for GUI integration and scripting.

## 5. Product Requirements

### Functional requirements

- FR-1: Enumerate available Bluetooth adapters.
- FR-2: Enumerate discovered devices without requiring the user to know a MAC
  address or platform-specific identifier.
- FR-3: Identify likely Switch-family controllers using OS metadata and
  observed HID descriptors, while allowing manual selection of an unknown
  device.
- FR-4: Execute pairing and connection operations through the native platform
  facility where possible.
- FR-5: Make state transitions observable: `Unknown`, `Discovered`,
  `Pairing`, `Paired`, `Connecting`, `Connected`, `HidReady`, `Disconnected`,
  and `Error`.
- FR-6: Verify input readiness independently from Bluetooth connection state.
- FR-7: Permit cancellation and timeouts for every operation that waits on
  hardware.
- FR-8: Provide actionable recovery guidance for adapter unavailable,
  permission denied, device busy, pairing rejected, timeout, and HID access
  failures.
- FR-9: Avoid duplicate pairing attempts when an operation is already active.
- FR-10: Export sanitized diagnostics in a versioned schema.
- FR-11: Keep platform-specific behavior behind a common capability interface.
- FR-12: Preserve unknown devices and unsupported states as diagnostic data
  rather than silently discarding them.

### Non-functional requirements

- NFR-1: Rust production code must remain compatible with the repository's
  `unsafe_code = "forbid"` policy.
- NFR-2: The core library must be deterministic and unit-testable without
  Bluetooth hardware by using fake platform backends.
- NFR-3: Hardware tests must be separated from ordinary CI and clearly marked
  as requiring a physical controller.
- NFR-4: Normal operation must not require administrator or root privileges
  except for explicitly documented virtual-device features.
- NFR-5: Logs must be structured, bounded, and redacted by default.
- NFR-6: No credentials, link keys, raw reports, or local machine paths may be
  written to tracked files.
- NFR-7: Long-running operations must have bounded memory use and explicit
  shutdown behavior.
- NFR-8: CLI output and the future GUI must use the same versioned domain
  events and error taxonomy.
- NFR-9: Public interfaces require documentation and focused tests.
- NFR-10: New dependencies must pass the existing format, lint, advisory,
  license, and supply-chain gates.

## 6. Architecture

The implementation should be layered so platform APIs and protocol research do
not leak into the user interface.

```text
CLI / GUI
    |
Application service: discovery, pairing workflow, reconnect, diagnostics
    |
Domain model: device identity, capabilities, state machine, events, errors
    |
Platform adapter: Windows Bluetooth/HID, Linux BlueZ/HID, macOS CoreBluetooth/HID
    |
Operating-system Bluetooth and HID facilities
```

Suggested Rust module boundaries:

- `domain`: identifiers, capabilities, states, events, errors, report model.
- `bluetooth`: adapter and device discovery abstractions.
- `pairing`: cancellable workflow and retry policy.
- `hid`: endpoint discovery, report capture, and decoder interfaces.
- `diagnostics`: redaction, schema versioning, export, and bounded capture.
- `platform`: feature-gated native backends.
- `cli`: argument parsing, rendering, exit codes, and JSON serialization.
- `gui`: deferred until the application service and event model are stable.

The application service owns orchestration and never calls OS APIs directly.
Platform backends return typed observations and errors. The domain layer must
not depend on a specific Bluetooth daemon, windowing toolkit, or controller
model.

### State model

State changes are validated by the domain state machine. Invalid transitions
produce an error event rather than being silently coerced. A connection can be
`Connected` while `HidReady` remains false. This distinction is required for
diagnosing controllers that pair but do not expose usable input.

### Capability model

Each platform backend reports capabilities such as discovery, pairing,
trust/bond management, HID enumeration, input reading, raw report capture, and
virtual output. Unsupported capabilities are explicit and visible to callers.

### Protocol model

Controller-specific decoding is a plugin-like registry of versioned decoders,
selected from sanitized descriptor and report evidence. Unknown reports remain
available to the diagnostic layer. A decoder must never assume that a report
length or button layout is stable without fixtures or hardware evidence.

## 7. Platform Strategy

| Platform | First responsibility | Expected constraints |
| --- | --- | --- |
| Linux in WSL2 | BlueZ discovery, pairing, HID inspection, fixtures | Bluetooth adapter passthrough and permissions may be required |
| Native Windows | Bluetooth pairing and HID connection | Native APIs are required for dependable host integration; WSL2 is not the production backend |
| macOS | Discovery, pairing, HID inspection | API and entitlement behavior must be validated on supported macOS versions |

Development starts with WSL2 for repeatable Linux-side inspection and native
Windows probes for host behavior. A USB Bluetooth adapter dedicated to WSL2
is preferred for experiments so the Windows host adapter is not disrupted.

The first platform milestone is not “all platforms work.” It is a shared
contract with one tested backend and honest capability reporting on the others.

## 8. Development Phases

### Phase 0: Lab and evidence log

- Record controller models, firmware versions when visible, host OS versions,
  adapter chipsets, and connection modes.
- Establish WSL2 setup and a repeatable adapter passthrough procedure.
- Define a sanitized observation format and store only fixtures, not personal
  machine identifiers or raw credentials.
- Test original Switch controllers as a protocol comparison where available.

Exit criteria: a contributor can reproduce discovery and collect a sanitized
diagnostic session on the reference Linux setup.

### Phase 1: Domain and fake backend

- Implement identifiers, capabilities, states, events, errors, and timeouts.
- Add a fake backend that simulates success, cancellation, and common failure
  paths.
- Define JSON schemas for command requests, events, and diagnostics.

Exit criteria: state transitions, error mapping, and CLI-facing contracts are
covered without hardware.

### Phase 2: Linux CLI prototype

- Implement BlueZ-backed discovery and pairing operations.
- Add HID endpoint verification and bounded report observation.
- Ship `adapters`, `scan`, `pair`, `connect`, `disconnect`, `info`, and
  `diagnose` in human and JSON modes.

Exit criteria: a known controller can be discovered, paired, connected, and
classified as HID-ready or not, with a useful diagnostic on failure.

### Phase 3: Windows backend

- Implement native Windows Bluetooth discovery and pairing.
- Implement native HID enumeration and input-readiness checks.
- Reuse the domain, application, diagnostics, and CLI layers.

Exit criteria: Windows can complete the same acceptance workflow for every
controller model claimed in the support matrix.

### Phase 4: macOS backend

- Implement the smallest supported discovery and pairing surface.
- Validate HID visibility and report access against real hardware.
- Document OS-version and permission constraints.

Exit criteria: macOS capability reporting is accurate and supported workflows
have hardware evidence.

### Phase 5: Controller decoding and output

- Add verified Switch 2 report decoders with fixture tests.
- Normalize buttons, sticks, triggers, battery, and connection events.
- Define the GameCube mapping and conflict behavior.
- Prototype virtual output only after the native input model is stable.

Exit criteria: a documented controller model produces repeatable normalized
events and a tested mapping suitable for the intended GameCube consumer.

### Phase 6: GUI

- Build a thin GUI over the application service and event stream.
- Provide adapter/device selection, guided pairing, connection status, live
  input test, profile selection, and diagnostic export.
- Keep advanced report details behind a developer-oriented diagnostics view.

Exit criteria: a new user can complete pairing without a terminal, while a
developer can reach the same evidence and error details.

## 9. Testing and Verification

### Unit tests

Test state transitions, timeout and cancellation behavior, retry policy,
redaction, schema compatibility, report parsing, and CLI exit-code mapping.

### Contract tests

Run every platform backend against the same fake-backend contract. Contract
tests must verify capability reporting, error categories, event ordering, and
shutdown behavior.

### Fixture tests

Store sanitized HID descriptors and report fixtures with provenance, model
label, and schema version. Fixtures must not contain credentials or uncontrolled
personal identifiers.

### Hardware matrix

Maintain a checked-in support matrix with columns for platform, OS version,
adapter, controller model, firmware visibility, discovery, pairing, HID
readiness, decoding, and known limitations. A blank cell means unverified, not
unsupported.

### CI gates

Every change continues to pass the existing formatting, lint, test, secret,
artifact, dependency, advisory, and supply-chain checks. Hardware jobs are
opt-in and never replace deterministic CI.

## 10. Security and Privacy

Bluetooth identifiers and HID data can be identifying and sensitive. The tool
must:

- redact or hash device addresses in normal logs;
- make raw report capture opt-in;
- avoid logging pairing material, link keys, or OS credential stores;
- write diagnostics only to an explicitly selected destination;
- avoid network access in the pairing process;
- validate output paths and refuse unsafe overwrite behavior by default;
- document any privileged operation needed for virtual output;
- keep all OS-specific permission prompts and trust operations visible to the
  user.

The diagnostic format must include a privacy level so an exported bundle can
be reviewed before sharing.

## 11. Support Policy

Support is evidence-based and versioned. A controller/platform combination is
`verified` only after the complete acceptance workflow succeeds on real
hardware. It may be marked `discovered`, `paired`, `connected`, or `input
verified` independently. Unsupported behavior must include the observed
failure and a suggested next diagnostic action.

The project should not claim that Bluetooth pairing alone makes a controller
usable in games. Input translation and virtual output are separate milestones.

## 12. Acceptance Criteria

The first usable release is complete when:

1. The CLI builds under the pinned Rust toolchain.
2. Fake-backend tests cover the full pairing state machine and expected error
   paths.
3. One reference Linux setup can discover and pair a verified controller.
4. The tool distinguishes Bluetooth connection from HID readiness.
5. A failed operation produces a bounded, sanitized diagnostic bundle.
6. JSON output is stable enough for a future GUI client.
7. Windows and macOS report their real capability boundaries rather than
   presenting unavailable actions as working.
8. Documentation includes the tested hardware matrix and WSL2 setup notes.
9. All repository hardening checks pass before the release is proposed.

## 13. Open Questions and Decision Log

These questions must be answered with experiments or authoritative platform
documentation before the corresponding phase is declared complete:

- Which Switch 2 controller models expose standard Bluetooth HID, and which
  require proprietary initialization or authentication?
- Does each target platform expose the same reports over Bluetooth and USB?
- Can Joy-Con units pair independently, and can they be combined reliably?
- Which Windows APIs provide stable pairing and HID access without a service?
- Which macOS APIs permit the required discovery and report inspection?
- Is a virtual GameCube device best implemented through an existing mapping
  layer, a platform-specific virtual HID device, or a dedicated adapter?
- What user-visible permissions are required for each output mode?

Each answer should be recorded as a short dated decision with hardware,
platform, evidence source, and confidence level. Unverified assumptions must
remain marked as such in the support matrix.

## 14. Immediate Implementation Backlog

1. Add the domain types and fake backend.
2. Add the versioned JSON event and diagnostic schemas.
3. Choose and document the Linux Bluetooth crate/API boundary after checking
   current dependency policy.
4. Build a read-only adapter and device inventory command.
5. Add the first sanitized HID fixture and hardware observation entry.
6. Implement pairing as a cancellable application workflow.
7. Run the repository's pre-commit and pre-push checks after each milestone.
