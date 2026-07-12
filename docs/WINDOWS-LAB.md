# Native Windows Controller Lab

This procedure collects sanitized evidence from the official Switch 2
GameCube controller on Windows 11. It does not authorize firmware, calibration,
pairing-key, bootloader, or other persistent controller writes.

## Session rules

- Keep raw captures and logs outside the repository in a user-selected working
  directory.
- Never record Bluetooth addresses, USB serial numbers, usernames, host paths,
  credentials, link keys, or pairing material in tracked files.
- Record discovery, pairing, connection, HID readiness, decoding, and output
  verification as independent states.
- Mark claims `unverified` until repeated against physical hardware.
- Stop before every hardware checkpoint listed in `IMPLEMENTATION-PLAN.md`.

## Sanitized observation template

Copy this template to a private lab note. Transfer only redacted conclusions to
the support matrix.

```text
Date:
Platform: Windows 11
Controller model: BEE-021
Connection: USB / Bluetooth
Operation:
Expected result:
Observed result:
Highest state reached:
Evidence confidence: unverified / observed once / repeated
Sanitized fixture reference:
Notes:
```

## Read-only USB baseline

The USB session may enumerate device interfaces, HID descriptors, report IDs,
report lengths, and input reports. It may compare SDL3-visible controls with
the project's observations. It must not send output reports until a later
checkpoint explicitly approves an evidence-backed volatile command.

The proposed first write experiment is documented in Decision 0003. It is a
single start-stream candidate followed by a bounded reply and ten-second input
observation. The candidate remains non-executable. Do not claim interface 1 or
run the experiment until the user explicitly approves that checkpoint. A
failure must stop the experiment; it does not authorize automatically sending
the rest of SDL's sequence.

After the reviewed one-, two-, and four-command probes failed to produce input,
custom packet escalation stopped. Use official current SDL3 for the working
wired baseline and normalized capability comparison. Do not translate SDL's
remaining unknown, rumble-related, or grip commands into project allowlisted
packets merely because the complete upstream driver uses them.

SDL wired support also depends on build configuration. The official generic
SDL 3.4.12 Windows x64 runtime tested in this lab used passive `If_Hid` even
when the official libusb DLL was available. A working SDL comparison requires
an SDL binary built with libusb HIDAPI support; merely placing libusb beside a
binary built without `HAVE_LIBUSB` is insufficient.

The controller's wired path is split across two interfaces. Use WinUSB
interface 1 for bounded initialization commands and replies, then release it.
Read continuous report ID `0x05` state from HID interface 0. Do not wait for
state reports on bulk IN; that endpoint carries command replies in SDL's model.

## Bluetooth session

The Bluetooth session begins with read-only adapter and device inventory. The
user must confirm the sanitized candidate before pairing. The application may
ask Windows to perform pairing but must not read, export, or persist link keys.
Pairing success does not imply HID readiness or decoded input.

## Recovery boundary

Normal Windows unpairing and Nintendo's documented SYNC or USB pairing process
are the supported recovery paths. Switch 2 re-pairing remains unverified until
console hardware is available. No recovery procedure may depend on modifying
controller firmware.
