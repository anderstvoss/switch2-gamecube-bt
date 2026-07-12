# Decision 0003: retain a one-packet USB input probe behind approval

- Date: 2026-07-11
- Status: accepted
- Current SDL source audited: `82141a2439acc111661102ba6f968c85e71cff40`

## Context

The current SDL Switch 2 USB path sends ten initialization packets after
read-only calibration and serial-number flash queries. Its comments identify
three packets as unknown, two as rumble setup, one as charging-grip setup, two
as feature-output configuration, one as selecting report format `0x05`, and
one as "Start output." It reads a reply after each packet but does not validate
the reply contents. The current source does not demonstrate which packets are
required specifically for BEE-021 input, nor whether any command is persistent.

SDL's public history provides useful narrowing evidence:

- [`a798da2e`](https://github.com/libsdl-org/SDL/commit/a798da2ec773a2b8166975ed3afa8072fe3d0d2a)
  introduced BEE-021 wired support through libusb.
- Immediately before
  [`70bfdd01`](https://github.com/libsdl-org/SDL/commit/70bfdd013a804fdb15ec906d4ba18389c57e9420),
  initialization consisted only of the 16-byte `0x03/0x91/0x0d` packet now
  described as "Start output." That commit replaced it with a full sequence
  captured from real hardware.
- [`82374b47`](https://github.com/libsdl-org/SDL/commit/82374b47784d73d705bd2398ee38ed3b1ac4c22f)
  later added explicit selection of report ID `0x05` and updated the decoder to
  require the resulting 64-byte layout.
- [`9fd3dbfc`](https://github.com/libsdl-org/SDL/commit/9fd3dbfc42a247b996858fe66fa835bdb1f03aa3)
  removed four sequence entries because they appeared to be console queries,
  not setup commands.
- [`ef993416`](https://github.com/libsdl-org/SDL/commit/ef99341691ad979d42e83cf4705eb107e90b2561)
  moved initialization after calibration reads to avoid a delay; this is an
  ordering optimization, not evidence that calibration reads enable input.

The earlier working path is evidence that the start-stream packet is the
smallest known input probe. Repetition during device initialization suggests
session scope, but neither comments nor commit history prove that it cannot
persist state. The name "Start output" means controller-to-host reporting in
this context; it is still a host-to-controller USB write.

## Decision

Model a one-command `candidate_minimal_input_probe` containing only the
start-stream packet. Keep it classified as `CandidateVolatile`, which makes the
normal preflight reject it before transport access. Do not export the internal
WinUSB transport factory or add a CLI execution path in this change.

The first live test, after this decision is reviewed and explicitly approved,
will be bounded as follows:

1. require a unique USB identity `057e:2073` and the already observed interface
   1 endpoint layout;
2. claim only interface 1 with a finite deadline;
3. send exactly the 16-byte start-stream candidate once;
4. read one bounded command reply and then observe bounded 64-byte input
   reports for no more than ten seconds;
5. send no cleanup command; release the interface by dropping the transport;
6. record only sanitized report IDs, lengths, counts, and success/failure; and
7. disconnect and reconnect USB before any later probe so session behavior is
   observable.

If the probe does not produce input, stop. Do not automatically add report
format, feature-output, unknown, rumble, grip, flash, calibration-write,
firmware, reset, or pairing commands. A subsequent packet requires a new
evidence review and approval.

## Consequences

The next live experiment can be substantially smaller than SDL's current full
sequence, but it is not yet authorized or executable. A successful probe would
support session-volatility and input-enablement claims for one packet; it would
not validate the remaining sequence, calibration, output features, or
Bluetooth behavior. A failed probe would establish only that more setup is
needed, not that the whole SDL sequence is safe.

## Observed result

The approved probe was run once on Windows 11. The controller returned a
12-byte reply to the single start-stream command, but produced no input reports
during the bounded ten-second observation. The interface was then released
without another command. See
`observations/2026-07-11-windows-usb-start-stream-probe.md`.

This validates the host transport and shows the one-packet candidate is
insufficient. The next smallest historically supported probe is report-format
`0x05` followed by start-stream on a freshly reconnected USB session. That
experiment remains limited to those two packets and must stop on failure.

The two-command experiment was then run on a freshly reconnected session.
Report-format `0x05` returned an 8-byte reply and start-stream returned a
12-byte reply, but no input reports arrived in ten seconds. See
`observations/2026-07-11-windows-usb-report5-probe.md`.

The next smallest reviewed probe adds the paired feature-output mask and enable
commands before report-format and start-stream. These are the remaining two
described, non-rumble commands in the modeled SDL sequence. They remain
candidate-volatile; unknown, rumble, grip, flash, firmware, reset, pairing, and
calibration-write commands remain excluded.

The four-command experiment was run after another USB reconnect. All commands
were acknowledged with reply lengths 12, 12, 8, and 12 bytes, but no input
reports arrived. See
`observations/2026-07-11-windows-usb-described-probe.md`.

This exhausts the described non-rumble candidates. Do not escalate the custom
initializer to SDL's remaining unknown, rumble-related, or grip commands.
Instead, validate wired behavior through official current SDL3 and treat its
driver as the maintained wired reference. Project-owned USB work remains a
controlled evidence and fixture path for Bluetooth decoding and the Windows
service, not a replacement for SDL3's application-facing wired support.
