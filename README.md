# switch2-gamecube-bt

Rust workspace for Switch 2 GameCube Bluetooth experiments.

## Project status (2026-11-09)

**Research/reference project; the standalone controller-driver roadmap is paused.**
This repository is **not a working Bluetooth gamepad driver** and is not
recommended for end-user controller support. Its useful results are the
BEE-021 (Switch 2 NSO GameCube controller) Windows USB input/protocol and
calibration research, plus preliminary Windows Bluetooth discovery diagnostics.
A Bluetooth GATT session, wireless input, and virtual-gamepad output have
**not** been implemented here.

Other projects now offer usable Switch 2 controller Bluetooth paths, while
upstream Linux HID support is advancing. Rather than duplicate that work,
preserve this repository as an auditable research baseline; resume a standalone
implementation only if it addresses a specific unsupported capability.
[Development specification](docs/DEVELOPMENT-SPEC.md) and
[implementation plan](docs/IMPLEMENTATION-PLAN.md) remain as **historical
plans**, not active commitments.

### Where to find working code

The following are **third-party, project-reported capabilities**, not verified
by this repository. Check each project's current README, releases, supported
controllers, and driver prerequisites before installing anything.

| Platform / use case | Project | Reported support / limitations |
| --- | --- | --- |
| **Windows 10/11 — Joy-Con 2, Pro Controller 2, NSO GameCube** | [Switch2Connect](https://github.com/TommyWabg/Switch2Connect) ([releases](https://github.com/TommyWabg/Switch2Connect/releases)) | BLE pairing and input with virtual-controller options; installation of the appropriate virtual-controller driver may be required. |
| **NSO GameCube (BEE-021) — Windows, Linux, macOS** | [NSO GameCube Controller Pairing App](https://github.com/RyanCopley/NSO-GameCube-Controller-Pairing-App) ([BLE protocol documentation](https://github.com/RyanCopley/NSO-GameCube-Controller-Pairing-App/blob/main/NSO_GC_BLE_PROTOCOL.md)) | GameCube-focused wired/wireless implementation; platform-specific setup and output restrictions apply (macOS does not provide its Xbox virtual-controller output). |
| **Linux — NSO GameCube and Pro Controller 2** | [switch2-controllers-linux](https://github.com/trevlars/switch2-controllers-linux) | Tested BLE userspace bridge with raw L2CAP/ATT, `uinput` gamepad output, and motion reporting; its Joy-Con 2 support is explicitly **untested/experimental**. |
| **Linux — Joy-Con 2 BLE research/driver** | [linux-switch2](https://github.com/novakpetya/linux-switch2) + [bluez-switch2](https://github.com/novakpetya/bluez-switch2) | Experimental kernel/BlueZ integration for Joy-Con 2 L/R BLE, alongside USB support for other models; requires out-of-tree components. |
| **Windows/Linux alternative bridge** | [pad2go](https://github.com/patrickdeangelis/pad2go) | Cross-platform BLE bridge with virtual-controller output; macOS supports motion/emulator output rather than a system-wide virtual gamepad. |

**Upstream Linux:** Switch 2 HID support was included in the
[Linux 7.4 HID merge-window pull request (October 2026)](https://lkml.iu.edu/2610.2/14674.html).
The initial upstream driver is **USB-focused and preliminary**: it does not
provide native Switch 2 BLE pairing/transport, and features such as IMU,
rumble, and battery remain future work. Do **not** interpret kernel integration
as complete wireless support; consult your distribution's actual kernel
version and current upstream development before relying on it.

The repository includes pinned local checks, committed git hook wrappers,
secret and artifact blockers, Rust lint/test gates, supply-chain checks,
CodeQL, OpenSSF Scorecard, SBOM generation, Dependabot, issue/PR templates, and
security reporting documentation.

## AI Usage Disclosure

This project was authored with extensive use of LLM/Coding Agents.

## Local Setup

Install the local tooling once per machine. Install `uv` using the official
installation method for your platform, then use it to install `pre-commit`:

Install gitleaks separately and make it available on `PATH`.

```bash
uv tool install pre-commit
cargo install cargo-deny cargo-audit
git config core.hooksPath .githooks
```

Then verify the gates:

```bash
pre-commit run --all-files
pre-commit run --all-files --hook-stage pre-push
```

The hook stack is documented in
[docs/pre-commit-hooks.md](docs/pre-commit-hooks.md).

The product and engineering direction is documented in
[docs/DEVELOPMENT-SPEC.md](docs/DEVELOPMENT-SPEC.md).
The ordered implementation roadmap is in
[docs/IMPLEMENTATION-PLAN.md](docs/IMPLEMENTATION-PLAN.md).
The native Windows hardware procedure is in
[docs/WINDOWS-LAB.md](docs/WINDOWS-LAB.md), and evidence-backed support claims
are tracked in [docs/SUPPORT-MATRIX.md](docs/SUPPORT-MATRIX.md).

## Diagnostic CLI status

The general `s2bt` controller-service commands (`scan`, `pair`, etc.)
still use a deterministic **fake backend**. They do not pair or connect to
physical hardware. Machine-readable output identifies simulated results with
`"backend": "fake"`. Separate Windows-specific commands implement real USB
diagnostics and read-only BLE discovery, but **not Bluetooth pairing or input**.

```bash
cargo run --bin s2bt -- scan
cargo run --bin s2bt -- --json scan
cargo run --bin s2bt -- pair fake-bee-021
```

On Windows, the following commands perform **read-only** USB discovery and
observation; other explicitly named USB experiments in this repository can
send reviewed session-initialization commands with user approval. See the
[support matrix](docs/SUPPORT-MATRIX.md) for the recorded wired input results:

```powershell
cargo run --bin s2bt -- --json usb-inventory
cargo run --bin s2bt -- --json usb-descriptor
cargo run --bin s2bt -- --json usb-observe --seconds 10 --limit 256
```

## Security

Please do not open public issues for vulnerabilities. Use GitHub private
vulnerability reporting for this repository, or contact the maintainer directly
if that is unavailable. See [SECURITY.md](SECURITY.md).

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
