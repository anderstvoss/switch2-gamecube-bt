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
