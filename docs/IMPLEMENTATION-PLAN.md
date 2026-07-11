# Switch 2 GameCube Bluetooth Implementation Plan

This plan turns [DEVELOPMENT-SPEC.md](DEVELOPMENT-SPEC.md) into an ordered
series of small, reviewable changes. The first usable outcome is a Linux/WSL2
CLI that can discover, pair, connect, and diagnose a controller. Windows and
macOS backends follow the shared contracts instead of creating separate
application flows.

## Working Rules

- Keep production Rust compatible with `unsafe_code = "forbid"`.
- Land one vertical slice at a time: domain contract, fake behavior, platform
  behavior, user-facing command, and tests.
- Keep Bluetooth hardware tests opt-in; deterministic tests remain the default.
- Record hardware observations as sanitized fixtures and support-matrix data.
- Do not add a dependency until its API boundary and policy impact are clear.
- Preserve the existing repository hardening checks on every change.

## Milestone 0: Development Lab

Goal: make hardware observations reproducible from the Windows host and WSL2.

Tasks:

1. Document the WSL2 distribution, Rust toolchain, required Linux packages,
   Bluetooth service expectations, and USB adapter passthrough procedure.
2. Add a hardware observation template containing platform, adapter,
   controller model, firmware visibility, operation, result, and evidence.
3. Add a support matrix with `unverified`, `discovered`, `paired`,
   `connected`, and `input verified` states.
4. Test an original Switch controller as a comparison device if available.

Deliverables:

- WSL2 setup notes in `docs/WSL2-LAB.md`.
- Sanitized support matrix in `docs/SUPPORT-MATRIX.md`.
- No raw addresses, credentials, logs, or machine-specific paths committed.

Exit gate: a contributor can repeat discovery and record an observation
without changing application code.

## Milestone 1: Domain Contracts

Goal: define the stable vocabulary shared by every backend and client.

Planned modules:

- `src/domain/identity.rs`: adapter and device identifiers.
- `src/domain/capability.rs`: supported operations and output modes.
- `src/domain/state.rs`: validated connection state machine.
- `src/domain/event.rs`: state, progress, input, and diagnostic events.
- `src/domain/error.rs`: user-safe error categories with developer context.
- `src/domain/mod.rs`: public exports and documentation.

Tasks:

1. Define opaque identifiers so platform-native handles do not leak into the
   application layer.
2. Define state transitions and reject invalid transitions.
3. Define operation options, deadlines, cancellation, and retry policy.
4. Define a versioned diagnostic model with privacy level and redaction rules.
5. Add unit tests for every valid transition and representative invalid paths.

Exit gate: domain tests pass without OS access, and the public types have
documentation suitable for a future GUI client.

## Milestone 2: Backend and Application Contracts

Goal: make orchestration testable before selecting all native APIs.

Planned modules:

- `src/platform/mod.rs`: `PlatformBackend` and capability contracts.
- `src/platform/fake.rs`: deterministic fake backend.
- `src/application/mod.rs`: pairing, reconnect, discovery, and diagnostics.

Tasks:

1. Define backend methods for adapters, discovery, pair, trust/bond,
   connect, disconnect, HID inspection, and bounded report observation.
2. Make unsupported operations explicit instead of silently falling back.
3. Implement fake success, timeout, cancellation, permission, pairing,
   connection, and HID-readiness failures.
4. Define event ordering and shutdown behavior.
5. Add contract tests that every backend must satisfy.

Exit gate: the complete pairing workflow can be tested against the fake
backend, including cancellation and all public error categories.

## Milestone 3: CLI Foundation

Goal: expose the application contract before hardware integration is complete.

Planned modules:

- `src/cli/args.rs`: command and option parsing.
- `src/cli/render.rs`: human-readable event rendering.
- `src/cli/json.rs`: versioned machine-readable output.
- `src/bin/s2bt.rs`: binary entry point.

Tasks:

1. Add `adapters`, `scan`, `pair`, `trust`, `connect`, `disconnect`, `info`,
   and `diagnose` commands.
2. Support bounded waits, cancellation, quiet mode, and JSON mode.
3. Map domain errors to stable exit-code categories.
4. Ensure human output is concise while JSON includes state and capability
   details needed by a GUI.
5. Test argument validation, rendering, JSON schema shape, and exit codes.

Exit gate: the CLI works end-to-end with the fake backend and has no platform
assumptions in command handling.

## Milestone 4: Linux/WSL2 Backend

Goal: complete the first hardware-backed vertical slice.

Planned modules:

- `src/platform/linux/mod.rs`: Linux backend wiring.
- `src/platform/linux/bluez.rs`: adapter, discovery, pairing, and connection.
- `src/platform/linux/hid.rs`: HID endpoint and report observation.
- `tests/linux_contract.rs`: backend contract coverage where practical.

Tasks:

1. Select the BlueZ API/crate after checking current Rust version,
   maintenance, licensing, and `cargo-deny` policy.
2. Implement read-only adapter and device inventory first.
3. Add pairing, trust/bond, connect, and disconnect with explicit timeouts.
4. Detect HID readiness separately from Bluetooth connection.
5. Add bounded report capture with default redaction and opt-in raw capture.
6. Run the workflow against reference hardware and add sanitized fixtures.
7. Document WSL2 permissions and adapter failure recovery.

Exit gate: the CLI can discover and pair a reference controller, report the
actual final state, and export a sanitized diagnostic on failure.

## Milestone 5: Protocol Evidence and Decoding

Goal: understand and normalize verified controller input.

Planned modules:

- `src/hid/report.rs`: validated report representation.
- `src/hid/decoder.rs`: decoder trait and registry.
- `src/controllers/`: model-specific decoders and mappings.
- `tests/fixtures/`: sanitized descriptors and reports.

Tasks:

1. Record descriptor and report metadata with provenance and confidence.
2. Implement unknown-report preservation before model-specific decoding.
3. Add decoders only for layouts supported by fixtures or repeatable hardware
   evidence.
4. Normalize buttons, sticks, triggers, battery, and connection events.
5. Test malformed, truncated, unexpected, and unknown reports.

Exit gate: at least one verified controller model produces stable normalized
events from fixtures and live hardware.

## Milestone 6: Windows Backend

Goal: provide native host integration for Windows users.

Planned modules:

- `src/platform/windows/mod.rs`: Windows backend wiring.
- `src/platform/windows/bluetooth.rs`: native discovery and pairing.
- `src/platform/windows/hid.rs`: native HID enumeration and reads.

Tasks:

1. Select the native API boundary and document required Windows versions.
2. Implement adapter/device inventory before mutating operations.
3. Implement pair, connect, disconnect, and HID readiness.
4. Reuse Linux contract tests through fake and mocked platform seams where
   native testing is unavailable.
5. Validate behavior from the Windows host independently of WSL2.

Exit gate: each claimed controller/platform combination has hardware evidence
for discovery, pairing, connection, and HID readiness.

## Milestone 7: macOS Backend

Goal: add macOS support with accurate capability boundaries.

Tasks:

1. Select the supported macOS API surface and minimum OS version.
2. Implement discovery and pairing within the permitted native workflow.
3. Validate HID access and report observation.
4. Document permission prompts, unsupported operations, and known limitations.

Exit gate: supported macOS workflows are hardware-verified and unavailable
operations are clearly reported by the CLI.

## Milestone 8: GameCube Output

Goal: translate normalized input into the intended GameCube consumer.

Tasks:

1. Define the GameCube mapping, dead zones, ranges, and conflict rules.
2. Decide whether output should use an existing mapping layer, virtual HID,
   or a dedicated platform adapter.
3. Implement output behind a capability-gated interface.
4. Document any administrator/root permissions and cleanup behavior.
5. Add integration tests for mapping and disconnect cleanup.

Exit gate: a verified controller produces repeatable GameCube-compatible
output, and disconnects never leave a stale virtual device behind.

## Milestone 9: GUI

Goal: provide a guided desktop workflow over the stable application service.

Screens/workflows:

- adapter and device selection;
- pairing progress and recovery;
- connection and HID-readiness status;
- live input test;
- controller profile and GameCube mapping;
- sanitized diagnostic export;
- developer diagnostics for report evidence.

Tasks:

1. Choose the GUI toolkit only after CLI event and JSON contracts stabilize.
2. Consume application events rather than calling platform APIs directly.
3. Ensure every GUI action has a CLI-equivalent operation.
4. Test cancellation, reconnect, failure recovery, and permission prompts.

Exit gate: a new user can pair a verified controller without a terminal, and a
developer can export the same evidence available from the CLI.

## Cross-Cutting Verification

For each milestone:

1. Add focused unit or contract tests with the change.
2. Run formatting, linting, and tests using the pinned toolchain.
3. Run the repository pre-commit checks for source and documentation changes.
4. Run pre-push checks before any release or publish proposal.
5. Update the support matrix, changelog, and relevant documentation.
6. Review the diff for secrets, raw captures, binary artifacts, and local paths.

## First Five Pull Requests

1. `docs: add WSL2 lab and support-matrix templates`
2. `feat: add domain identifiers, capabilities, states, and errors`
3. `feat: add backend contract, fake backend, and application workflow`
4. `feat: add s2bt CLI with human and JSON output`
5. `feat: add Linux inventory and pairing backend`

Each pull request should remain independently reviewable and leave the tree
passing all available deterministic checks.
