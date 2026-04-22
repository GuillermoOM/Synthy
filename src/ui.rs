#[cfg(target_arch = "x86_64")]
use eframe::egui;
#[cfg(target_arch = "x86_64")]
use egui_plot::{Line, Plot, PlotPoints};
#[cfg(target_arch = "x86_64")]
use crossbeam_channel::Sender;

#[cfg(target_arch = "x86_64")]
use crate::state::AudioCommand;

#[cfg(target_arch = "x86_64")]
pub struct SynthApp {
    tx: Sender<AudioCommand>,
    // Local state to reflect UI changes
    frequency: f32,
    volume: f32,
    fm_index: f32,
    octave: i32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    waveform: f32, // 0: Sine, 1: Triangle, 2: Saw, 3: Square
    lfo_rate: f32,
    note_playing: bool,
    next_voice: u8,
    active_keys: [Option<u8>; 10],
    // Keep audio stream alive
    _audio_stream: cpal::Stream,
}

#[cfg(target_arch = "x86_64")]
impl SynthApp {
    pub fn new(tx: Sender<AudioCommand>, audio_stream: cpal::Stream) -> Self {
        Self {
            tx,
            frequency: 440.0,
            volume: 0.2, // Lowered to prevent polyphony saturation
            fm_index: 0.0,
            octave: 4,
            attack: 0.1,
            decay: 0.1,
            sustain: 0.8,
            release: 0.2,
            waveform: 0.0,
            lfo_rate: 1.0,
            note_playing: false,
            next_voice: 0,
            active_keys: [None; 10],
            _audio_stream: audio_stream,
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard input (Keys 1-0 for Notes)
        let base_c = 16.3516 * 2.0_f32.powi(self.octave);
        let keys = [
            (egui::Key::Num1, 0),  // C
            (egui::Key::Num2, 2),  // D
            (egui::Key::Num3, 4),  // E
            (egui::Key::Num4, 5),  // F
            (egui::Key::Num5, 7),  // G
            (egui::Key::Num6, 9),  // A
            (egui::Key::Num7, 11), // B
            (egui::Key::Num8, 12), // C (next octave)
            (egui::Key::Num9, 14), // D
            (egui::Key::Num0, 16), // E
        ];

        ctx.input(|i| {
            for (idx, (key, semitones)) in keys.iter().enumerate() {
                let freq = base_c * 2.0_f32.powf(*semitones as f32 / 12.0);
                
                // Only trigger if NOT already active to prevent auto-repeat clipping!
                if i.key_pressed(*key) && self.active_keys[idx].is_none() {
                    let voice = self.next_voice;
                    self.next_voice = (self.next_voice + 1) % 5;
                    self.active_keys[idx] = Some(voice);
                    let _ = self.tx.send(AudioCommand::NoteOn(voice, freq));
                    // Also update global frequency display
                    self.frequency = freq;
                    let _ = self.tx.send(AudioCommand::UpdateFrequency(freq));
                }
                if i.key_released(*key) {
                    if let Some(voice) = self.active_keys[idx] {
                        let _ = self.tx.send(AudioCommand::NoteOff(voice));
                        self.active_keys[idx] = None;
                    }
                }
            }
        });

        // Ensure UI continuously redraws so animations/plots update smoothly
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Synthy Testing UI (Use keys 1-0 to play notes)");
            
            ui.horizontal(|ui| {
                // Left Panel: Controls (fixed width)
                ui.allocate_ui_with_layout(
                    egui::vec2(250.0, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.group(|ui| {
                            ui.label("Global Settings");
                            if ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume")).changed() {
                                let _ = self.tx.send(AudioCommand::UpdateVolume(self.volume));
                            }
                            ui.add(egui::Slider::new(&mut self.octave, 1..=8).text("Octave"));
                            if ui.add(egui::Slider::new(&mut self.fm_index, 0.0..=10.0).text("FM Index")).changed() {
                                let _ = self.tx.send(AudioCommand::UpdateFMIndex(self.fm_index));
                            }
                            if ui.add(egui::Slider::new(&mut self.lfo_rate, 0.1..=20.0).text("LFO Rate")).changed() {
                                let _ = self.tx.send(AudioCommand::UpdateLFORate(self.lfo_rate));
                            }
                            if ui.add(egui::Slider::new(&mut self.frequency, 20.0..=2000.0).text("Frequency").logarithmic(true)).changed() {
                                let _ = self.tx.send(AudioCommand::UpdateFrequency(self.frequency));
                                if self.note_playing {
                                    // Also update the active test note frequency
                                    let _ = self.tx.send(AudioCommand::NoteOn(0, self.frequency));
                                }
                            }
                        });

                        ui.group(|ui| {
                            ui.label("Waveform");
                            let mut wf_changed = false;
                            wf_changed |= ui.radio_value(&mut self.waveform, 0.0, "Sine").changed();
                            wf_changed |= ui.radio_value(&mut self.waveform, 1.0, "Triangle").changed();
                            wf_changed |= ui.radio_value(&mut self.waveform, 2.0, "Sawtooth").changed();
                            wf_changed |= ui.radio_value(&mut self.waveform, 3.0, "Square").changed();
                            
                            if wf_changed {
                                let _ = self.tx.send(AudioCommand::UpdateWaveform(self.waveform));
                            }
                        });

                        ui.group(|ui| {
                            ui.label("ADSR Envelope");
                            if ui.add(egui::Slider::new(&mut self.attack, 0.001..=2.0).text("Attack")).changed() {
                                let _ = self.tx.send(AudioCommand::UpdateAttack(self.attack));
                            }
                            if ui.add(egui::Slider::new(&mut self.decay, 0.001..=2.0).text("Decay")).changed() {
                                let _ = self.tx.send(AudioCommand::UpdateDecay(self.decay));
                            }
                            if ui.add(egui::Slider::new(&mut self.sustain, 0.0..=1.0).text("Sustain")).changed() {
                                let _ = self.tx.send(AudioCommand::UpdateSustain(self.sustain));
                            }
                            if ui.add(egui::Slider::new(&mut self.release, 0.001..=2.0).text("Release")).changed() {
                                let _ = self.tx.send(AudioCommand::UpdateRelease(self.release));
                            }
                        });

                    }
                );

                // Right Panel: Plots (flexible, taking remaining space)
                ui.vertical(|ui| {
                    let total_height = ui.available_height();
                    let half_height = (total_height - 30.0) / 2.0; // Subtract spacing
                    
                    ui.group(|ui| {
                        ui.label("ADSR Plot");
                        Plot::new("adsr_plot")
                            .height(half_height)
                            .include_y(0.0)
                            .include_y(1.1)
                            .allow_drag(false)
                            .allow_scroll(false)
                            .show(ui, |plot_ui| plot_ui.line(
                                Line::new(PlotPoints::new({
                                    let hold_time = 0.5;
                                    vec![
                                        [0.0, 0.0],
                                        [self.attack as f64, 1.0],
                                        [(self.attack + self.decay) as f64, self.sustain as f64],
                                        [(self.attack + self.decay + hold_time) as f64, self.sustain as f64],
                                        [(self.attack + self.decay + hold_time + self.release) as f64, 0.0]
                                    ]
                                })).name("ADSR Envelope")
                            ));
                    });
                    
                    ui.group(|ui| {
                        ui.label("Waveform Shape");
                        Plot::new("waveform_plot")
                            .height(ui.available_height() - 10.0) // Take remaining height
                            .include_y(1.0)
                            .include_y(-1.0)
                            .allow_drag(false)
                            .allow_scroll(false)
                            .show(ui, |plot_ui| {
                                let window = 0.02; // 20ms time window
                                let points: PlotPoints = (0..1000).map(|i| {
                                    let t = (i as f64 / 1000.0) * window;
                                    let f = self.frequency as f64;
                                    let fm_idx = self.fm_index as f64;
                                    let vol = self.volume as f64;
                                    
                                    let mod_phase = t * f * fm_idx * std::f64::consts::TAU;
                                    let modulator = mod_phase.sin();
                                    
                                    let carrier_phase = t * f * std::f64::consts::TAU + (modulator * 200.0 / f.max(1.0));
                                    
                                    let mut phase = carrier_phase % std::f64::consts::TAU;
                                    if phase < 0.0 { phase += std::f64::consts::TAU; }
                                    let normalized_phase = phase / std::f64::consts::TAU;
                                    
                                    let mut y = match self.waveform as u8 {
                                        0 => phase.sin(),
                                        1 => {
                                            if normalized_phase < 0.5 { -1.0 + 4.0 * normalized_phase }
                                            else { 3.0 - 4.0 * normalized_phase }
                                        },
                                        2 => 2.0 * normalized_phase - 1.0,
                                        3 => {
                                            if normalized_phase < 0.5 { 1.0 } else { -1.0 }
                                        },
                                        _ => 0.0,
                                    };
                                    
                                    y *= vol;
                                    [t, y]
                                }).collect();
                                
                                plot_ui.line(Line::new(points).name("Waveform"));
                            });
                    });
                });
            });
        });
    }
}
