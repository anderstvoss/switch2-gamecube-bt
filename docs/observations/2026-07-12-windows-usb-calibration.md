# Windows USB read-only calibration: BEE-021

- Date: 2026-07-12
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: documented factory/user calibration reads
- State reached: calibration parsed and applied in memory
- Evidence confidence: observed once

## Sanitized result

The project read seven documented 64-byte calibration blocks through bounded
bulk transfers: factory gyro bias, left and right stick calibration, factory
accelerometer bias, trigger zero points, and optional left/right user stick
overrides. All seven parsed successfully. Factory calibration was valid; no
left or right user override was present.

The serial-number block was deliberately not requested. Calibration bytes were
used only in process memory to normalize subsequent HID frames and were then
discarded. No calibration contents, serial, device path, raw reply, or machine
identifier was retained.

## Interpretation

SDL stores stick minimum and maximum values as spans from neutral, not absolute
endpoints. The parser follows that semantics and applies calibrated centers and
spans to the portable wired decoder. A subsequent 4,096-frame calibrated
capture remained near center; full stick travel was not observed in that run and
is not claimed as verified.
