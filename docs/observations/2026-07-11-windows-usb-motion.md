# Windows USB motion input: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- Operation: sanitized six-axis motion exercise
- State reached: motion input verified
- Evidence confidence: observed once

## Sanitized result

Motion samples were structurally present immediately after wired stream
initialization but remained nearly static during physical movement. After the
controller had produced stable sensor timestamps, the project reapplied the
reviewed feature-enable value `0x27` once and received a 12-byte reply.

During the next bounded physical exercise, 4,096 of 4,096 decoded frames
contained motion samples. All acceleration and angular-velocity axes changed
substantially during roll, pitch, yaw, and translation. No raw report or
calibration value was retained.

## Interpretation

SDL's motion byte order is verified for BEE-021. Feature enable must be applied
after sensor timestamp warm-up to obtain live motion; applying the same value
only during initial startup was insufficient in this session.

Motion scale and bias remain provisional until read-only factory calibration is
implemented. The successful ranges verify responsiveness and axis presence,
not final calibrated accuracy.
