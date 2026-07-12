# Windows packaged BLE baseline

Date: 2026-07-12
Platform: Windows 11 native host
Controller: not placed in SYNC mode
Connection: Bluetooth Low Energy advertisement observation only

## Result

A locally signed MSIX capability host was initially started by its installed
executable path. That method did not prove package identity and is superseded
by the registered activation result below. The adapter-status command reported
both Bluetooth Low Energy and the central role as supported, and its sanitized
result-file channel completed successfully.

A subsequent two-second packaged BLE advertisement scan, performed without a
controller action, completed successfully and returned zero advertisements.
This run validates package launch, result retrieval, and the bounded watcher
lifecycle. It is not evidence about BEE-021 discoverability.

The experiment did not pair, connect, perform GATT service discovery, or send a
controller command. Package, certificate, staging, and result artifacts remain
local-only and ignored.

## Supervised SYNC attempt

The user subsequently held SYNC as an eight-second scan started from the
installed executable path. The watcher completed normally and returned zero
advertisements. Because package identity was not verified for that launch, this
result is retained as an execution observation only and is not BLE discovery
evidence.

No candidate was available to confirm. The experiment did not pair, connect,
perform GATT service discovery, or send a controller command.

## Registered activation baseline

The capability host was updated to include a package-identity status command
and a local launch helper that uses registered Windows application activation.
That helper reported `package_identity_present=true`. A controller-free,
two-second BLE scan then completed through the same registered path and
returned zero advertisements. This validates package identity, sanitized result
retrieval, and watcher lifecycle for the next supervised experiment.

## Next checkpoint

Run the prebuilt, registered-activation eight-second advertisement scan while
the user places BEE-021 in SYNC mode. If a sanitized candidate appears, stop
for confirmation before any connection or GATT access.
