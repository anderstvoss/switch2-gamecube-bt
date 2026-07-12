# Windows USB decoded input matrix: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: sanitized decoded button and axis exercise
- State reached: button and axis input verified
- Evidence confidence: observed once

## Sanitized result

After exact SDL-reference initialization, the project decoded 4,096 valid
64-byte report ID `0x05` frames from HID interface 0. During a physical exercise
it observed every modeled button:

`A`, `B`, `X`, `Y`, `Start`, `Home`, `Capture`, `C`, D-pad up/down/left/right,
`L`, `R`, `Z`, and `ZL`.

Both sticks and both analog triggers changed across substantial bidirectional
normalized ranges. Exact per-device calibration has not yet been read, so the
current fallback mapping intentionally does not claim calibrated centers or
full-scale endpoints.

No raw report, device path, serial number, calibration content, pairing
material, or machine identifier was retained.

## Interpretation

SDL's pinned button offsets and packed 12-bit stick layout are correct for the
observed BEE-021. Wired buttons, D-pad, sticks, and analog triggers are ready
for fixture-backed decoding. Motion and read-only calibration remain separate
verification items.
