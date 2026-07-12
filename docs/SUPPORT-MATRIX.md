# Controller Support Matrix

This matrix records sanitized evidence. Use one row per platform, connection,
adapter label, controller model, and operation combination.

Do not enter Bluetooth addresses, USB serial numbers, usernames, host paths,
credentials, raw logs, or packet captures.

## State definitions

| State | Meaning |
| --- | --- |
| `unverified` | The combination has not yet been tested. |
| `discovered` | The relevant Bluetooth stack lists the adapter or controller. |
| `paired` | Pairing or bonding completed successfully. |
| `connected` | A live controller connection was established. |
| `input verified` | Sanitized input observations were received and interpreted. |

Use the highest state actually demonstrated. Do not infer a later state from an
earlier one.

## Matrix

| Platform | Environment | Connection | Adapter label | Controller model | Operation | State | Confidence | Evidence reference | Date | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Windows 11 | Native host | USB | n/a | BEE-021 | inventory | discovered | observed once | `observations/2026-07-11-windows-usb-inventory.md` | 2026-07-11 | Read-only HID metadata only. |
| Windows 11 | Native host | USB | n/a | BEE-021 | bulk endpoint inventory | discovered | observed once | `observations/2026-07-11-windows-usb-bulk-inventory.md` | 2026-07-11 | Interface 1 descriptor has one 64-byte bulk endpoint in each direction; no claim or transfer. |
| Windows 11 | Native host | USB | n/a | BEE-021 | passive input observation | discovered | observed once | `observations/2026-07-11-windows-usb-passive-observation.md` | 2026-07-11 | HID handle opened; no reports before initialization. |
| Windows 11 | Native host | USB | n/a | BEE-021 | one-packet start-stream probe | connected | observed once | `observations/2026-07-11-windows-usb-start-stream-probe.md` | 2026-07-11 | 12-byte command reply; no input reports in ten seconds. |
| Windows 11 | Native host | USB | n/a | BEE-021 | report-format plus start-stream probe | connected | observed once | `observations/2026-07-11-windows-usb-report5-probe.md` | 2026-07-11 | 8- and 12-byte replies; no input reports in ten seconds. |
| Windows 11 | Native host | USB | n/a | BEE-021 | four described-command probe | connected | observed once | `observations/2026-07-11-windows-usb-described-probe.md` | 2026-07-11 | Replies were 12, 12, 8, and 12 bytes; no input reports. |
| Windows 11 | Native host | USB | official SDL 3.4.12 x64 | BEE-021 | wired SDL baseline | discovered | repeated | `observations/2026-07-11-sdl3-windows-runtime-baseline.md` | 2026-07-11 | Generic runtime fell back to `If_Hid`; no state changes, including with libusb DLL present. |
| Windows 11 | Native host | USB | project WinUSB + HID | BEE-021 | exact SDL initialization and HID readiness | input verified | observed once | `observations/2026-07-11-windows-usb-sdl-sequence.md` | 2026-07-11 | Ten command replies followed by continuous 64-byte report ID `0x05` on HID interface 0. |
| Windows 11 | Native host | USB | project WinUSB + HID | BEE-021 | decoded buttons, sticks, and triggers | input verified | observed once | `observations/2026-07-11-windows-usb-decoded-input.md` | 2026-07-11 | All 16 modeled buttons and all six axes observed across 4,096 frames; calibration pending. |
| Windows 11 | Native host | USB | project WinUSB + HID | BEE-021 | six-axis motion | input verified | observed once | `observations/2026-07-11-windows-usb-motion.md` | 2026-07-11 | All six motion axes responsive after post-warm-up feature enable; calibration pending. |
| Windows 11 | Native host | Bluetooth | `<adapter label>` | BEE-021 | discovery | unverified | unverified | `<lab note>` | `<YYYY-MM-DD>` | |
| WSL2 | `<distribution>` | Bluetooth | `<adapter label>` | BEE-021 | discovery | unverified | unverified | `<lab note>` | `<YYYY-MM-DD>` | |
| Linux | Native host | Bluetooth | `<adapter label>` | BEE-021 | discovery | unverified | unverified | `<lab note>` | `<YYYY-MM-DD>` | |

## Observation template

Copy this template into a private lab note before adding a sanitized result to
the matrix:

```text
Platform/environment:
Connection: USB / Bluetooth
Adapter label:
Controller model:
Operation: discovery / pairing / connection / input verification
Expected result:
Observed result:
State reached:
Evidence confidence: unverified / observed once / repeated
Evidence reference:
Date:
Notes:
```

Evidence references must point to sanitized notes, not committed logs or raw
captures.
