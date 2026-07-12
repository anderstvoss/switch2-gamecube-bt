# Windows Bluetooth discovery attempt

Date: 2026-07-12
Platform: Windows 11 native host
Controller: BEE-021
Connection: Bluetooth, controller placed in SYNC mode by the user

## Result

Windows reported that a default Bluetooth adapter was available. The initial
known-device inventory returned no controller candidates, which is expected
for an unpaired controller and is not treated as a controller failure.

Two bounded discovery attempts then returned no candidates. The first used an
unpaired classic-Bluetooth device selector. The second used a Bluetooth Classic
association-endpoint watcher, which is the endpoint kind Windows uses for
managed pairing. Neither attempt paired, connected to, or sent a command to the
controller.

A later prebuilt eight-second `PairTool` active-discovery run also returned no
Bluetooth Classic endpoint while the user pressed SYNC immediately after the
scan began. The first `PairTool` attempt is discarded because compilation
delayed the scan start; the prebuilt continuous attempt is the valid result.
This narrows the gap to BEE-021 discoverability versus the Windows Bluetooth
stack. It does not establish a controller defect or a Windows defect.

## Timing and confidence

The user observed that the controller's SYNC mode lasts approximately eight to
ten seconds. The scanner is correspondingly bounded to at most ten seconds.
Nintendo's public pairing instructions require holding SYNC for at least one
second but do not state an advertising timeout. The timeout is therefore not
classified as a hardware limitation or a Windows limitation yet. A controller
firmware timeout is plausible, but remains an inference rather than evidence.

## Next investigation

Compare the association-endpoint watcher with the Windows Bluetooth picker or
another supported Windows discovery surface. Do not attempt pairing until a
sanitized candidate has been observed and confirmed by the user.
