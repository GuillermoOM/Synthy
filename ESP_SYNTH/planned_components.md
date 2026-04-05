# Planned Components: Synthy Hardware

This document outlines the intended hardware components and their assigned functions for the final version of the Synthy embedded synthesizer.

## 1. Control Interface

| Assigned Function | Recommended Component | Specification / Value |
| :--- | :--- | :--- |
| **Master Volume** | Rotary Potentiometer | 10k Ohm Logarithmic (Audio) Taper |
| **Envelope: Attack** | Slide Potentiometer (Fader) | 10k Ohm Linear Taper |
| **Envelope: Decay** | Slide Potentiometer (Fader) | 10k Ohm Linear Taper |
| **Envelope: Sustain** | Slide Potentiometer (Fader) | 10k Ohm Linear Taper |
| **Envelope: Release** | Slide Potentiometer (Fader) | 10k Ohm Linear Taper |
| **Filter: Cutoff** | Rotary Potentiometer | 10k Ohm Linear Taper |
| **Filter: Resonance** | Rotary Potentiometer | 10k Ohm Linear Taper |
| **LFO: Rate** | Rotary Potentiometer | 10k Ohm Linear Taper |
| **LFO: Depth / Routing** | Rotary Potentiometer | 10k Ohm Linear Taper |
| **FM: Modulation Amount** | Rotary Potentiometer | 10k Ohm Linear Taper |
| **Menu & Sample Navigation** | Rotary Encoder | Incremental with built-in push switch |
| **Wavetable Index Sweep** | Rotary Encoder | Incremental with built-in push switch |
| **Oscillator / Wave Select** | Rotary Encoder | Incremental with built-in push switch |
| **Musical Keyboard Keys** | Mechanical Switch | Tactile style (e.g., Cherry MX Brown) |
| **Octave Up / Down** | Tactile Push Button | Standard momentary switch |

## 2. Technical Requirements
- **ADC Multiplexing:** To handle 10+ potentiometers on a single ESP32, an external multiplexer (like the CD74HC4067, 16-channel) is highly recommended.
- **Key Matrix:** A diode-protected matrix should be used for the keyboard to avoid ghosting and minimize GPIO usage.
- **Power Management:** The system should handle 5V USB-C input and provide regulated 3.3V for the ESP32 and DAC.
