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
- Continue autonomously through implementation, documentation, tests, and
  non-controller experiments. Pause only when a physical controller action is
  required, state that action plainly, and resume when the user confirms it.

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

The bounded transport implementation now exists behind an internal Windows
adapter boundary. It validates endpoint direction, packet size, nonzero
deadlines, exact writes, bounded nonempty reads, and redacts platform transfer
failures into stable timeout or failure categories. Its interface-claiming
factory is intentionally unused and not exported by the Windows module, so no
production or CLI path can create it. No live interface claim or transfer has
been performed. The next review gate is deciding which minimum volatile input
initialization slice has sufficient evidence for a user-approved live test.

SDL history shows that its earliest BEE-021 wired path used only the 16-byte
start-stream packet, before a later change adopted a full sequence captured
from real hardware. Later history added report-format selection and removed
four apparent console queries. This makes the start-stream packet the smallest
evidence-backed input probe, but does not prove persistence behavior. The code
models it as `CandidateVolatile`, so preflight still rejects it before I/O. See
Decision 0003 for sources, uncertainty, the bounded experiment plan, and the
mandatory approval gate.

The one-packet probe received a 12-byte command reply but no input reports in
ten seconds. This confirms the WinUSB command/reply path and rules out
start-stream alone for the current 64-byte input mode. After a fresh USB
reconnection, the next bounded probe will select report format `0x05` and then
start streaming. It will stop without adding feature-output, unknown, rumble,
grip, flash, firmware, reset, pairing, or calibration-write commands if input
still does not appear.

The fresh-session report-format plus start-stream probe received ordered 8- and
12-byte replies but no input reports. The next bounded experiment adds the
paired feature-output mask and enable commands ahead of those two accepted
commands. This exhausts the four described non-rumble initialization
candidates; failure will trigger a new evidence review rather than automatic
use of SDL's unknown or output-related steps.

The four described commands were acknowledged with reply lengths 12, 12, 8,
and 12 bytes, but still produced no input. Custom initialization stops here:
the remaining SDL steps are unknown, rumble-related, or grip-specific. The next
wired baseline will use official current SDL3 directly, since SDL3 already
provides maintained Switch 2 USB support. Project-owned USB transport remains
useful for bounded protocol evidence, fixtures, and the future background
service; it is not intended to replace SDL3 for ordinary wired applications.

Official SDL 3.4.12 source contains the wired bulk driver, but the official
generic Windows x64 runtime tested here enumerated only passive `If_Hid` input.
Adding the official libusb DLL did not change that because SDL's libusb HIDAPI
path must be compiled with `SDL_HIDAPI_LIBUSB`/`HAVE_LIBUSB`; it cannot be added
to an existing DLL at runtime. A custom SDL build and the project's WinUSB
transport would execute the same audited protocol. The next controlled baseline
therefore uses the exact pinned SDL sequence through WinUSB as an isolated
upstream-reference experiment, without promoting its unknown or output-related
packets into the normal allowlist.

The exact SDL sequence produced all ten bounded replies. SDL source then
clarified the split transport: interface 1 bulk endpoints carry initialization
and replies, while interface 0 HID carries state. A same-session HID check
received continuous 64-byte report ID `0x05` input. Wired HID readiness is now
verified. The next work is deterministic BEE-021 decoding from SDL's documented
layout followed by a sanitized physical input matrix; no raw reports will be
committed.

The first decoded physical matrix processed 4,096 valid frames and observed all
16 modeled buttons plus both sticks and both analog triggers. SDL's button and
packed-axis offsets are verified for this controller. Generic axis
normalization remains provisional until read-only factory/user calibration is
implemented. Motion verification is the next wired checkpoint.

Motion is now verified across all three acceleration and all three
angular-velocity axes. The feature-enable value `0x27` must be reapplied after
sensor timestamp warm-up; doing so yielded responsive motion in all 4,096
observed frames. Scale and bias remain provisional until read-only calibration
is incorporated.

Read-only calibration is now implemented and verified: seven documented blocks
parse successfully, factory calibration is valid, and no user stick override is
present. The serial-number block is skipped, and calibration bytes never cross
the process boundary. The calibrated decoder centers values near zero; full
stick endpoint verification remains outstanding because physical travel was not
captured in the latest run.

### Goal 4: Native Windows Bluetooth Low Energy

The BEE-021 wireless path is now treated as Bluetooth Low Energy (BLE), not
standard Bluetooth Classic HID. Public Switch 2 reverse-engineering work
describes a proprietary BLE `0x91` GATT protocol, and a Linux project reports
NSO GameCube support through a BLE bridge. These are external leads, not yet
project-verified protocol facts.

Implement read-only Windows BLE advertisement discovery first. Retain only
sanitized names, short rotating-identifier digests, and evidence-backed service
matches; do not persist Bluetooth addresses or link keys. Then establish a BLE
GATT client connection and independently verify service discovery, session
initialization, input notifications, and disconnect behavior.

Windows reports that the current adapter supports BLE and the central role, but
the unpackaged diagnostic received no advertisements. Microsoft documents the
BLE advertisement API's `bluetooth` package capability. Add a thin, Windows-only
MSIX capability host before treating an empty scan as controller evidence; keep
the Rust scanner and protocol model package-agnostic. Generating a test
certificate, installing an MSIX package, or changing Windows developer policy
remains a separate user-approved checkpoint.

The authorized local MSIX capability host is now installed with the current
result-file support. Packaged adapter status succeeded, and a controller-free
two-second packaged scan completed successfully with zero advertisements. This
validates package launch, sanitized result retrieval, and watcher lifecycle;
the next checkpoint is a prebuilt eight-second packaged scan while the user
places BEE-021 in SYNC mode.

The first supervised scan was started by installed executable path, so package
identity was not proven and its empty result is not BLE discovery evidence. The
capability host now has a package-identity diagnostic and is launched through
registered Windows application activation; that path proved package identity
and completed a controller-free watcher baseline. The next physical checkpoint
is one prebuilt eight-second scan through that verified path while the user
places BEE-021 in SYNC mode.

The verified supervised scan completed normally with package identity present
but returned zero advertisements. This is valid negative evidence for the
current Windows BLE advertisement watcher only. Audit public protocol leads and
materially different Windows discovery mechanisms before requesting another
physical checkpoint; do not pair or connect speculatively.

The audit identifies Windows' unpaired Bluetooth-LE device-selector watcher as
a distinct, documented discovery path. It is being added as a bounded,
read-only diagnostic and must be validated without the controller before one
more supervised SYNC attempt. Public BlueZ work establishes the shared Switch 2
vendor-service UUID as a protocol lead, but is explicitly developed and tested
only with Pro Controller 2; it must not be treated as BEE-021 initialization
evidence.

The verified supervised device-selector scan also completed normally with zero
devices. The advertisement watcher and the unpaired device-selector watcher
are now independent negative Windows discovery evidence. Audit the available
Windows implementation lead before choosing another physical experiment.

That audit found that the available Windows implementation uses an ESP32-S3 BLE
bridge, not the Windows Bluetooth stack. This is an external hardware and
architecture alternative, not evidence that a further host-only Windows scan
will discover BEE-021. Do not obtain, flash, configure, or connect such a
bridge without explicit approval.

WSL2 is not in scope. The next connection path is an ESP32-S3 bridge running a
separate BLE stack. No compatible bridge is attached to the current host. Build
a project-owned, read-only serial bridge diagnostic first; obtaining or
flashing bridge hardware and connecting BEE-021 remain user-approved hardware
checkpoints.

The previous known-device, association-endpoint, and active PairTool scans are
preserved as negative Bluetooth Classic evidence only. They do not test BLE and
must not be interpreted as a controller or Windows hardware failure. The
observed SYNC window remains approximately eight to ten seconds; its cause is
unverified.

After requesting SYNC mode and confirming a sanitized BLE candidate, add a
cancellable GATT connection and independent service, notification, and input
readiness checks. Do not access or persist link keys.

Exit gate: Windows reports BLE discovery, GATT connection, service readiness,
notification readiness, and decoded input as separate evidence-backed states.

### Goal 5: BLE decoding and safe outputs

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
