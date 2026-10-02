# chip8

A CHIP-8 interpreter written in Rust, built from scratch as an exercise in low-level systems programming — implemented directly from the CowGod ref. specification, without reference to an existing emulator.

## Features
- Complete 35 instructions
- 16 key hex keypad
- 64x32 display gui via minifb
- buzzer via rodio
- 6 configurable quirks

## Test Results

Verified against roms in roms directory which provided from [Timendus's Chip8 Test Suite](https://github.com/Timendus/chip8-test-suite)

| Test | Result |
|---|---|
| 1 — CHIP-8 logo | pass |
| 2 — IBM logo | pass |
| 3 — corax+ | pass |
| 4 — flags | pass |
| 5 — quirks (CHIP-8 mode) (when configured with right quirks) | 6/6 |
| 5 — quirks (SUPER-CHIP mode) (when configured with right quirks) | 6/6 |

## Controls
The 16 key CHIP-8 keypad is mapped to the top-left block of the keyboard.
```
|chip8|          |keyboard|
|1 2 3 C|        |1 2 3 4|
|4 5 6 D|  ->    |Q W E R|
|7 8 9 E|        |A S D F|
|A 0 B F|        |Z X C V|
```
## Running
```
cargo run --release -- <rom-file>
```

ROMs are read from the `roms/` directory, so only the file name is needed (including the file extension).

720 Instructions are read per second. ('CPU_FREQ')
Timers and display update at 60hz .

## References

- [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Timendus' CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite)
