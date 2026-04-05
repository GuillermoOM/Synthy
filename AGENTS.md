# Synthy: Embedded Digital Synthesizer Context

## Project Goals
Building a headless, embedded digital synthesizer.
- **Current Development**: Local Fedora Distrobox container (x86_64).
- **Final Target**: Raspberry Pi Zero W (Single-core ARMv6, 1GHz, 512MB RAM).

## Hardware Architecture
- **Audio Engine**: Raspberry Pi Zero W (running Synthy).
- **Audio Output**: I2S DAC connected via GPIO on the Pi.
- **Interface/Display**: Secondary microcontroller handling keyboard matrix, knobs, and SPI display (connected via Serial/UART).

## Software Stack
- **Language**: Rust.
- **Audio I/O**: `cpal` (ALSA/Host interface).
- **DSP Graph**: `fundsp` (Allocation-free synthesis).
- **Communication**: `crossbeam-channel` (Lock-free UI-to-Audio thread messaging).
- **State/Presets**: `serde` and `bincode`.

## Strict Real-Time Coding Rules (The "Synthy Rules")
These rules MUST be followed in all code generation for the audio thread:
1. **NO HEAP ALLOCATIONS**: No `Box`, `Vec`, `String`, or `.clone()` on dynamic structures inside the audio callback.
2. **NO BLOCKING**: No `std::sync::Mutex`, no File I/O, no network calls, and no blocking channel operations.
3. **LOCK-FREE COMMUNICATION**: Use `crossbeam-channel` or `heapless` SPSC queues for passing parameters to the audio thread.
4. **ZERO-COST ABSTRACTIONS**: Prioritize performance for the 1GHz single-core target.
5. **OPTIMIZED MATH**: Favor wavetables, polynomial approximations (e.g., PolyBLEP), and efficient filter coefficients over complex standard library math calls.
