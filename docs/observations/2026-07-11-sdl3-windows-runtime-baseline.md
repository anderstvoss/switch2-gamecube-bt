# SDL 3.4.12 Windows runtime baseline: BEE-021

- Date: 2026-07-11
- Platform: Windows 11 native host
- Connection: USB
- Controller model: BEE-021
- SDL release: 3.4.12 official x64 runtime
- libusb release: 1.0.30 official x64 runtime
- State reached: enumerated through passive HID, input not ready
- Evidence confidence: repeated

## Sanitized result

The official SDL 3.4.12 Windows x64 runtime enumerated one `057e:2073`
joystick, labeled it `If_Hid`, and exposed 21 buttons and four axes. During a
bounded 20-second exercise, SDL reported no button activity and every axis
remained zero. Repeating enumeration with the official libusb 1.0.30 x64 DLL
on the temporary process search path produced the same passive result.

No runtime or test binary was installed system-wide or added to the
repository. Temporary paths, raw data, serial numbers, and machine identifiers
were not retained.

## Build finding

SDL's Switch 2 source driver calls `SDL_InitLibUSB` and uses libusb bulk
transfers on interface 1. SDL's build defines that implementation only when
`SDL_HIDAPI_LIBUSB` results in `HAVE_LIBUSB`; dynamic loading merely controls
how an already compiled implementation obtains libusb at runtime. The official
generic Windows DLL tested here does not activate that compiled path, so adding
`libusb-1.0.dll` afterward cannot turn its passive HID enumeration into the
Switch 2 bulk driver.

This reconciles the observations: SDL source supports wired Switch 2
controllers, while a particular binary must be built with libusb HIDAPI
support. Steam's working wired behavior is consistent with a libusb-capable
build, but Steam's transport remains a black box.

## Consequence

The project does not need to recreate SDL's normalized wired API. It does need
either a custom libusb-enabled SDL build or its existing WinUSB transport to
obtain controlled raw protocol evidence. The next experiment may reproduce the
exact pinned SDL initialization sequence through WinUSB as an isolated
upstream-reference test; those packets must not silently enter the normal
project allowlist.
