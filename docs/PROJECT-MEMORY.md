# Project Memory

This file is the durable context for ongoing development of native Windows
support for the official Nintendo Switch 2 GameCube controller, model BEE-021.
It is intentionally sanitized: do not add Bluetooth addresses, serials, raw
packets, certificates, package locations, usernames, machine paths, logs, or
generated binaries.

## Product goal

Deliver a Windows-native Bluetooth driver experience for BEE-021: automatic
reconnect, broadly compatible virtual gamepad output, and a full-fidelity API
for every evidence-verified control, motion data, battery, LEDs, rumble, and
unknown reports. Keep controller protocol, decoding, mapping, and application
contracts portable to Linux and future Switch 2 controllers.

No controller firmware, bootloader state, flash, calibration writes, pairing
keys, link keys, or undocumented persistent storage may ever be written.

## Working agreement

- Continue autonomously through code, tests, documentation, research, and
  non-controller experiments.
- Stop only for a physical controller action, a material design decision, PR
  review/merge, permission to change Windows state, or another genuine blocker.
- Coordinate physical tests explicitly: prebuild before requesting SYNC because
  the observed controller pairing window is roughly 8–10 seconds.
- Commit sensible, narrow packets; open draft PRs for meaningful milestones,
  not every small edit; wait for each PR to merge before beginning the next
  milestone.
- PR descriptions must explain what changed, why, hardware evidence, safety,
  validation, limits, and the next step. Keep old PR documentation consistent.
- Preserve `unsafe_code = "forbid"`, SHA-pin workflow actions, and keep all
  local hooks, security checks, dependency checks, formatting, Clippy, and tests
  passing before publishing work.

## Architecture decisions

- Rust workspace remains safe and platform-neutral. Windows, Linux/BlueZ, USB,
  BLE, SDL3, and future controllers are adapters around portable contracts.
- The future virtual gamepad uses a narrowly scoped KMDF/VHF component; the
  Rust workspace must not contain kernel or controller-specific unsafe logic.
- SDL3 is the wired implementation/reference and validation source; this project
  owns Bluetooth/BLE transport, session setup, decoding, diagnostics, and the
  native Windows virtual device path.
- Extra controls must not be forced into fixed XInput slots. Preserve them in
  the full-fidelity API until a verified Windows mapping is available.
- Windows package capability hosting is intentionally thin. It declares Windows
  BLE permissions but must not own controller protocol, raw-log persistence,
  pairing-key handling, or virtual HID code.

## Wireless transport reassessment

The initial plan assumed standard Bluetooth Classic HID pairing. That is no
longer the working hypothesis.

Current evidence strongly favors a proprietary Bluetooth Low Energy GATT path:

- BEE-021 radio documentation lists Bluetooth BR/EDR and Bluetooth LE support.
- Public Switch 2 reverse-engineering work describes a proprietary BLE `0x91`
  GATT protocol, with command, acknowledgement, and input-notification roles.
- Public work reports NSO GameCube controller support through BLE bridges.
- The public BlueZ Switch 2 plugin is an important protocol lead, but only its
  stated controller coverage is evidence. Audit and independently reproduce all
  behavior before treating it as BEE-021 fact.

Therefore Bluetooth Classic inventory, association-endpoint, and PairTool scans
are retained as negative evidence for the wrong transport, not evidence of
controller hardware failure. Do not continue the Windows Bluetooth picker or
Classic HID pairing path as the primary implementation.

## Verified wired baseline

- Windows sees BEE-021 over USB as Nintendo `057e:2073`, HID interface 0.
- The controller has a separate WinUSB bulk interface 1 for initialization
  command replies; continuous state arrives from HID interface 0.
- The exact current SDL reference initialization sequence produced ten bounded
  replies and continuous 64-byte report ID `0x05` state reports.
- Decoder evidence covers all 16 modeled buttons, both sticks, both analog
  triggers, and six-axis motion.
- Read-only factory and optional user calibration parsing is implemented. Seven
  documented blocks parsed successfully; factory calibration was valid; user
  stick overrides were absent; the serial-number block is deliberately skipped.
- A calibrated input exercise reached full normalized stick travel and observed
  all six motion axes. A later trigger exercise observed both trigger ranges.
- Trigger inputs are verified, but their decoder currently uses fallback 8-bit
  normalization rather than the stored trigger-zero calibration. Do not claim
  final trigger endpoint calibration yet.
- Official generic SDL Windows runtime lacked the libusb-enabled HIDAPI build
  path, so simply placing a libusb DLL beside it did not activate this controller.
  The project WinUSB + HID reference path is the reliable wired lab baseline.

## Bluetooth Classic evidence

- Default Windows Bluetooth adapter is present.
- Windows known-device inventory returned no candidate while BEE-021 was in
  SYNC mode.
- A bounded Windows Bluetooth Classic association-endpoint watcher returned no
  candidate.
- Microsoft PairTool is available and reports Bluetooth Classic support.
- A valid prebuilt, continuous eight-second PairTool active discovery scan also
  returned no Classic endpoint while SYNC was pressed.
- No Classic experiment paired, connected, opened HID, sent a controller command,
  or changed controller state.

Conclusion: Classic discovery is not the BEE-021 route to pursue.

## BLE evidence and current limitation

- The Windows adapter reports `low_energy_supported=true` and
  `central_role_supported=true`.
- The unpackaged BLE advertisement watcher received zero advertisements, so it
  is not valid controller evidence. It may reflect Windows app-capability scope.
- Microsoft documents the `bluetooth` package capability for its BLE
  advertisement APIs. The next comparison must run the scanner under package
  identity before another controller conclusion is drawn.
- A local test MSIX capability host has been built, signed with a local test
  certificate, and successfully installed. It declares only Bluetooth device
  capability and full-trust launch support.
- Package-launch stdout is not relayed by Windows. A `--result-file` option was
  added to write only sanitized CLI output to a caller-selected local file.
- The test package must be rebuilt with an incremented package version after
  executable or manifest changes before it can replace the installed version.
- Local package version `0.1.0.1` now contains the result-file support. Packaged
  adapter status succeeded, and a controller-free two-second packaged BLE scan
  completed with zero advertisements. This validates the result channel and
  watcher lifecycle but is not controller discovery evidence.
- A user-supervised packaged eight-second scan completed normally while the
  user held SYNC, but returned zero advertisements. Treat this as negative
  evidence for that specific scan only; no candidate was available to confirm,
  and no pairing, connection, GATT access, or controller command occurred.
- Test certificate, MSIX output, staging directory, and package artifacts are
  local-only and ignored. Do not commit or expose them.

## Immediate next steps

1. Audit the Windows packaged-launch and BLE watcher assumptions against
   authoritative platform behavior before interpreting the empty supervised
   scan or requesting another physical attempt.
2. Audit public Switch 2 BLE protocol implementations before adding a bounded,
   read-only GATT service-discovery client. Add session commands only after
   evidence, fixtures, and explicit user approval.
3. Request another supervised SYNC attempt only after the audit identifies a
   materially different discovery experiment.

## Package-host checkpoint

The user explicitly authorized creation and installation of the local test MSIX
capability host and its machine-level test-certificate trust. The user must be
asked again before removing the test package/certificate, enabling test-signing,
installing a kernel driver, or making any controller physical change.

## Important local files

- `docs/IMPLEMENTATION-PLAN.md`: authoritative ordered roadmap.
- `docs/DEVELOPMENT-SPEC.md`: product and engineering constraints.
- `docs/WINDOWS-LAB.md`: Windows hardware procedure and safety gates.
- `docs/SUPPORT-MATRIX.md`: only sanitized, evidence-backed claims.
- `docs/decisions/0004-windows-ble-capability-host.md`: package capability-host
  boundary.
- `docs/observations/`: sanitized hardware findings only.

## Recent merged milestones

- #16 wired BEE-021 input baseline.
- #17 read-only calibration.
- #18 Windows Bluetooth inventory.
- #19 independent adapter detection fix.
- #20 bounded Windows Bluetooth Classic discovery.
- #21 Windows PairTool lab-status diagnostic.
- #22 bounded PairTool active discovery.
- #23 Windows BLE discovery diagnostics and BLE architecture pivot.

## External research leads

- Nintendo support documents holding SYNC for at least one second and USB
  re-pairing to Switch 2; it does not document the observed timeout.
- Microsoft BLE advertisement documentation requires Bluetooth app capability.
- BlueZ Switch 2 BLE plugin: proprietary `0x91` GATT protocol, service and
  characteristic-role lead; independently validate BEE-021 applicability.
- Public Switch 2 BLE input viewer: useful handle/feature-flag lead, not a
  production authority.
- Public Linux NSO GameCube BLE bridge: useful compatibility claim and fixture
  lead, requiring audit before adoption.
