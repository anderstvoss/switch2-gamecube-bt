# WSL2 Bluetooth Lab

This guide defines a repeatable Windows-host and WSL2 procedure for collecting
sanitized Bluetooth and HID observations. It is a lab procedure, not an
application runtime requirement.

## Host prerequisites

- Windows 11 or a supported Windows 10 build with WSL2 enabled.
- A WSL2 distribution, preferably Ubuntu LTS.
- Rust through `rustup`, using the repository toolchain in
  `rust-toolchain.toml`.
- A Bluetooth adapter that can be passed through to WSL2 when the host adapter
  is not exposed to Linux.
- `usbipd-win` when USB passthrough is required.
- The repository's local security tools documented in `README.md`.

Keep Windows and WSL2 observations separate. A controller may be visible to the
Windows Bluetooth stack while remaining unavailable to the Linux stack.

## WSL2 packages

Inside the WSL2 distribution, install the diagnostic tools needed by the lab:

```bash
sudo apt update
sudo apt install --yes bluez bluez-tools usbutils
```

Confirm the tools are available:

```bash
bluetoothctl --version
lsusb --version
uname -a
```

If the distribution does not run a system service manager, start the Bluetooth
service using the distribution's supported method or perform observations
against the host stack instead. Do not commit service logs or machine-specific
configuration.

## Optional USB adapter passthrough

From PowerShell, identify the adapter:

```powershell
usbipd list
```

If passthrough is needed, use the adapter's displayed bus identifier:

```powershell
usbipd bind --busid <busid>
usbipd attach --wsl --busid <busid>
```

Inside WSL2, verify that the adapter is visible:

```bash
lsusb
bluetoothctl list
```

Detach the adapter when the observation session is complete:

```powershell
usbipd detach --busid <busid>
```

Never record USB serial numbers, Bluetooth addresses, hostnames, usernames, or
raw capture files in the repository. Record only sanitized observations in the
support matrix.

## Observation procedure

For each session:

1. Record the platform, WSL2 distribution, adapter identity label, and date in
   the support matrix.
2. Record whether the adapter is discovered by the relevant stack.
3. Use a known comparison controller first when one is available.
4. Record discovery, pairing, connection, and input verification separately.
5. Record the exact command or UI operation and its result.
6. Redact addresses, serial numbers, usernames, host paths, and credentials
   before sharing an observation.

Pairing success does not establish that input reports are usable. Keep the
`paired`, `connected`, and `input verified` states distinct.

## Exit criteria

The lab milestone is complete when another contributor can repeat discovery and
record a sanitized observation without changing application code.
