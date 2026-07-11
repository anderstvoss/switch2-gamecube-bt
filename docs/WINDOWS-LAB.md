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
