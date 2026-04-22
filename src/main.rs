mod audio;
mod dsp;
mod state;
#[cfg(target_arch = "x86_64")]
mod ui;

use crossbeam_channel::{unbounded, Sender};
use std::time::Duration;
use std::io::{BufRead, BufReader};
use crate::state::AudioCommand;

pub fn start_synth_backend() -> Result<(cpal::Stream, Sender<AudioCommand>), Box<dyn std::error::Error>> {
    // 1. Create communication channel between main and audio threads
    let (tx, rx) = unbounded();

    // 2. Setup audio stream
    let stream = audio::setup_audio_stream(rx)?;
    println!("Audio stream started successfully.");

    // 3. Setup Serial Interface for Microcontroller
    let port_name = "/dev/ttyUSB0"; // Default for Lolin32 Lite (CH340)
    let baud_rate = 115200;

    println!("Attempting to open serial port: {} at {} baud", port_name, baud_rate);
    match serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open() 
    {
        Ok(port) => {
            let tx_serial = tx.clone();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(port);
                let mut line = String::new();
                let mut current_freq = 440.0;

                loop {
                    line.clear();
                    if reader.read_line(&mut line).is_ok() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() { continue; }
                        println!("Received from serial: {}", trimmed);
                        if trimmed.starts_with('P') {
                            // Potentiometer: P<id>:<normalized_val>
                            let parts: Vec<&str> = trimmed[1..].split(':').collect();
                            if parts.len() == 2 {
                                if let (Ok(id), Ok(val)) = (parts[0].parse::<u8>(), parts[1].parse::<f32>()) {
                                    match id {
                                        0 => { // Frequency (20Hz - 2000Hz)
                                            let freq = 20.0 + val * 1980.0;
                                            current_freq = freq;
                                            let _ = tx_serial.send(AudioCommand::UpdateFrequency(freq));
                                        }
                                        1 => { // FM Index (0 - 10.0)
                                            let idx = val * 10.0;
                                            let _ = tx_serial.send(AudioCommand::UpdateFMIndex(idx));
                                        }
                                        2 => { // Volume (0.0 - 1.0)
                                            let _ = tx_serial.send(AudioCommand::UpdateVolume(val));
                                        }
                                        3 => { let _ = tx_serial.send(AudioCommand::UpdateAttack(val * 2.0)); }
                                        4 => { let _ = tx_serial.send(AudioCommand::UpdateDecay(val * 2.0)); }
                                        5 => { let _ = tx_serial.send(AudioCommand::UpdateSustain(val)); }
                                        6 => { let _ = tx_serial.send(AudioCommand::UpdateRelease(val * 2.0)); }
                                        7 => { let _ = tx_serial.send(AudioCommand::UpdateLFORate(val)); }
                                        _ => {}
                                    }
                                }
                            }
                        } else if trimmed.starts_with('B') {
                            // Button: B<id>:<state>
                            let parts: Vec<&str> = trimmed[1..].split(':').collect();
                            if parts.len() == 2 {
                                if let (Ok(id), Ok(state)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                                    match (id, state) {
                                        (0..=4, 1) => { 
                                            // Map button id to voice index
                                            let ratio = match id {
                                                0 => 1.0/2.0,
                                                1 => 3.0/4.0,
                                                2 => 1.0,
                                                3 => 5.0/4.0,
                                                4 => 3.0/2.0,
                                                _ => 1.0,
                                            };
                                            let _ = tx_serial.send(AudioCommand::NoteOn(id, current_freq * ratio)); 
                                        }
                                        (0..=4, 0) => { 
                                            let _ = tx_serial.send(AudioCommand::NoteOff(id)); 
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        } else if trimmed.starts_with('W') {
                            // Waveform: W:<index>
                            if let Ok(idx) = trimmed[2..].parse::<u8>() {
                                let _ = tx_serial.send(AudioCommand::UpdateWaveform(idx as f32));
                            }
                        }
                    }
                }
            });
            println!("Serial thread started.");
        }
        Err(e) => {
            eprintln!("Failed to open serial port {}: {}. Serial control disabled.", port_name, e);
        }
    }

    Ok((stream, tx))
}

#[cfg(target_arch = "x86_64")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Synthy: Embedded Digital Synthesizer (Local UI Mode) ---");
    println!("Hardware Target: x86_64 Local");
    
    let (stream, tx) = start_synth_backend()?;
    
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Synthy Testing UI",
        options,
        Box::new(|_cc| Ok(Box::new(ui::SynthApp::new(tx, stream)))),
    ).map_err(|e| e.into())
}

#[cfg(not(target_arch = "x86_64"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Synthy: Embedded Digital Synthesizer (Embedded Mode) ---");
    println!("Hardware Target: Pi Zero W (ARMv6)");
    println!("DSP Stack: CPAL + Fundsp");

    let (_stream, _tx) = start_synth_backend()?;

    // 4. Main Loop (Keep synth active)
    println!("Synthesizer Active!");
    println!("Lolin32 Lite HID Layout:");
    println!("- Pot P0 (GPIO32): Frequency (20-2000Hz)");
    println!("- Pot P1 (GPIO33): FM Modulation Index");
    println!("- Pot P2 (GPIO34): Master Volume");
    println!("- Pot P3-P6: ADSR (A, D, S, R)");
    println!("- Buttons B0-B4: Play Polyphonic Notes (5 Voices)");
    println!("- Encoder (GPIO16/17): Waveform Select (Sine, Tri, Saw, Sqr)");
    println!("Press Ctrl+C to exit.");

    loop {
        // Just keep the main thread alive
        std::thread::sleep(Duration::from_secs(1));
    }
}
