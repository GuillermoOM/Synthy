# Synthy Project Roadmap

Building a high-performance, real-time embedded digital synthesizer for Raspberry Pi Zero W.

## ✅ Completed Milestones

### 1. Core Foundation
- **Rust Audio Engine**: Established `cpal` host and `fundsp` graph architecture.
- **Real-Time Safety**: Implemented strict no-allocation/no-blocking rules in the audio callback.
- **Inter-Thread Communication**: Lock-free parameter updates using `crossbeam-channel` and `Shared` atomic handles.
- **Serial Protocol**: Defined a lightweight text-based protocol (`P<id>:<val>`, `B<id>:<state>`, `W:<idx>`) for MCU-to-Host communication.

### 2. Voice Architecture & DSP
- **FM Synthesis**: Implementation of a carrier/modulator pair with adjustable depth.
- **5-Voice Polyphony**: Parallel synthesis graph supporting 5 simultaneous notes.
- **Custom ADSR**: Created `SharedAdsr` with parameter locking at `NoteOn` to prevent envelope warping during playback.
- **Optimized Oscillators**: Developed `SharedOscillator` to handle 4 waveforms (Sine, Triangle, Saw, Square) in a single branchless node.
- **Anti-Click Logic**: Implemented phase-resetting on Note-On to ensure zero-crossing transients.
- **Global LFO**: Added a master LFO (controllable via Pot 7) to modulate the FM Index for "growl" and vibrato effects.

### 3. Hardware Integration (HID)
- **ESP32 Firmware**: PlatformIO project for Lolin32 Lite reading 8 potentiometers, 5 buttons, and 1 EC11 encoder.
- **State Synchronization**: MCU now transmits current physical knob positions immediately upon connection to sync the software state.
- **Signal Conditioning**: Implemented EMA (Exponential Moving Average) filtering and hysteresis on the MCU to prevent jitter.

### 4. Developer Experience (DX)
- **VS Code Integration**: Custom `tasks.json` and `launch.json` for seamless Rust debugging using LLDB.
- **Code Optimization**: Audited and reduced CPU usage by replacing parallel summing trees with efficient selector nodes.

---

## 🚀 Upcoming Roadmap

### 5. Advanced DSP (Next Priority)
- [ ] **Resonant Low-Pass Filter**: Implementation of a Moog-style ladder filter or efficient 2-pole state-variable filter.
- [ ] **Effects Chain**: Addition of a simple delay or reverb (using pre-allocated buffers).
- [ ] **Wavetable Support**: Integrating wavetable synthesis for more complex timbres.

### 6. Hardware Expansion
- [ ] **SPI LCD Menu**: Implementing the display driver on the ESP32 to visualize waveforms and parameter pages.
- [ ] **Keyboard Matrix**: Support for a larger physical keybed.
- [ ] **I2S DAC Configuration**: Finalizing ALSA settings for the PCM5102A/similar DAC on the Pi Zero.

### 7. Connectivity & Storage
- [ ] **Preset Management**: Saving and loading patches (`.patch` files) using `serde`.
- [ ] **USB-MIDI**: Configuring the Pi Zero as a MIDI Gadget for external controller support.
- [ ] **Web UI**: Optional WiFi-based dashboard for advanced configuration.
