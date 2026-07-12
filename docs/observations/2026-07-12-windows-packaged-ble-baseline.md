# Windows packaged BLE baseline

Date: 2026-07-12
Platform: Windows 11 native host
Controller: not placed in SYNC mode
Connection: Bluetooth Low Energy advertisement observation only

## Result

A locally signed MSIX capability host launched the existing safe-Rust
diagnostic with Windows package identity and the `bluetooth` device capability.
The packaged adapter-status command reported both Bluetooth Low Energy and the
central role as supported. Its sanitized result-file channel completed
successfully.

A subsequent two-second packaged BLE advertisement scan, performed without a
controller action, completed successfully and returned zero advertisements.
This run validates package launch, result retrieval, and the bounded watcher
lifecycle. It is not evidence about BEE-021 discoverability.

The experiment did not pair, connect, perform GATT service discovery, or send a
controller command. Package, certificate, staging, and result artifacts remain
local-only and ignored.

## Next checkpoint

Run the prebuilt packaged eight-second advertisement scan while the user places
BEE-021 in SYNC mode. If a sanitized candidate appears, stop for confirmation
before any connection or GATT access.

## Supervised SYNC attempt

The user subsequently held SYNC as the prebuilt packaged eight-second scan
started. The watcher completed normally and returned zero advertisements. This
is negative evidence for this specific Windows packaged BLE scan, not evidence
of a controller defect or proof that BEE-021 does not advertise over BLE.

No candidate was available to confirm. The experiment did not pair, connect,
perform GATT service discovery, or send a controller command.
