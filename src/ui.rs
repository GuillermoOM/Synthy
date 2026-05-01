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
    lfo_rate: f32,
    lfo_enabled: bool,
    fm_enabled: bool,
    filter_cutoff: f32,
    filter_resonance: f32,
    filter_enabled: bool,
    octave: i32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    waveform: f32, // 0: Sine, 1: Triangle, 2: Saw, 3: Square
    next_voice: u8,
    active_keys: [Option<u8>; 10],
    active_voices: [Option<f32>; 5],
    // Keep audio stream alive
    _audio_stream: cpal::Stream,
}

#[cfg(target_arch = "x86_64")]
impl SynthApp {
    pub fn new(tx: Sender<AudioCommand>, audio_stream: cpal::Stream) -> Self {
        Self {
            tx,
            frequency: 440.0,
            volume: 1.0, // Lowered to prevent polyphony saturation
            fm_index: 0.0,
            lfo_rate: 1.0,
            lfo_enabled: false,
            fm_enabled: false,
            filter_cutoff: 20000.0,
            filter_resonance: 0.707,
            filter_enabled: false,
            octave: 4,
            attack: 0.1,
            decay: 0.1,
            sustain: 0.8,
            release: 0.2,
            waveform: 0.0,
            next_voice: 0,
            active_keys: [None; 10],
            active_voices: [None; 5],
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
                    // "Mix" logic: If this note is already playing (e.g. in release phase), 
                    // find it and stop it first to "reset" the voice.
                    for v_idx in 0..5 {
                        if let Some(active_f) = self.active_voices[v_idx] {
                            if (active_f - freq).abs() < 0.1 {
                                let _ = self.tx.send(AudioCommand::NoteOff(v_idx as u8));
                                self.active_voices[v_idx] = None;
                                // We don't break, in case multiple voices are somehow playing it
                            }
                        }
                    }

                    // Standard voice allocation
                    let mut assigned_voice = None;
                    for offset in 0..5 {
                        let v = (self.next_voice + offset) % 5;
                        let is_held = self.active_keys.iter().any(|&opt| opt == Some(v));
                        if !is_held {
                            assigned_voice = Some(v);
                            self.next_voice = (v + 1) % 5;
                            break;
                        }
                    }

                    let voice = assigned_voice.unwrap_or_else(|| {
                        let v = self.next_voice;
                        self.next_voice = (v + 1) % 5;
                        for old_owner in self.active_keys.iter_mut() {
                            if *old_owner == Some(v) {
                                *old_owner = None;
                            }
                        }
                        v
                    });

                    self.active_keys[idx] = Some(voice);
                    self.active_voices[voice as usize] = Some(freq);
                    let _ = self.tx.send(AudioCommand::NoteOn(voice, freq));
                    self.frequency = freq;
                }
                
                if i.key_released(*key) {
                    if let Some(voice) = self.active_keys[idx] {
                        let _ = self.tx.send(AudioCommand::NoteOff(voice));
                        self.active_keys[idx] = None;
                        self.active_voices[voice as usize] = None;
                    }
                }
            }
        });

        // Ensure UI continuously redraws so animations/plots update smoothly
        ctx.request_repaint();

        egui::SidePanel::left("controls_panel")
            .exact_width(350.0)
            .show(ctx, |ui| {
                ui.heading("Synthy Testing UI (Keys 1-0)");
                ui.separator();
                
                // Helper to add scroll-to-adjust behavior to sliders
                let scroll_slider = |ui: &mut egui::Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, text: &str, log: bool| -> bool {
                    let mut slider = egui::Slider::new(value, range.clone()).text(text);
                    if log { slider = slider.logarithmic(true); }
                    let res = ui.add(slider);
                    if res.hovered() {
                        let scroll = ui.input(|i| i.raw_scroll_delta.y);
                        if scroll != 0.0 {
                            if log {
                                let factor = if scroll > 0.0 { 1.1 } else { 0.9 };
                                *value = (*value * factor).clamp(*range.start(), *range.end());
                            } else {
                                let step = (range.end() - range.start()) / 100.0;
                                *value = (*value + (if scroll > 0.0 { step } else { -step })).clamp(*range.start(), *range.end());
                            }
                            return true;
                        }
                    }
                    res.changed()
                };

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Oscillator").heading());
                                ui.separator();
                                
                                let mut oct_f = self.octave as f32;
                                if scroll_slider(ui, &mut oct_f, 1.0..=8.0, "Octave", false) {
                                    self.octave = oct_f as i32;
                                }

                                if scroll_slider(ui, &mut self.frequency, 20.0..=2000.0, "Frequency", true) {
                                    let _ = self.tx.send(AudioCommand::UpdateFrequency(self.frequency));
                                }
                                
                                ui.horizontal(|ui| {
                                    ui.label("Waveform:");
                                    let mut wf_changed = false;
                                    wf_changed |= ui.radio_value(&mut self.waveform, 0.0, "Sine").changed();
                                    wf_changed |= ui.radio_value(&mut self.waveform, 1.0, "Triangle").changed();
                                    wf_changed |= ui.radio_value(&mut self.waveform, 2.0, "Saw").changed();
                                    wf_changed |= ui.radio_value(&mut self.waveform, 3.0, "Square").changed();
                                    
                                    if wf_changed {
                                        let _ = self.tx.send(AudioCommand::UpdateWaveform(self.waveform));
                                    }
                                });
                            });
                            
                            ui.add_space(5.0);

                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Modulation (FM & LFO)").heading());
                                ui.separator();
                                let mut fm_changed = false;
                                fm_changed |= ui.checkbox(&mut self.fm_enabled, "Enable FM Modulator").changed();
                                
                                let mut lfo_changed = false;
                                lfo_changed |= ui.checkbox(&mut self.lfo_enabled, "Enable Global LFO").changed();
                                
                                if fm_changed || lfo_changed {
                                    let actual_fm = if self.fm_enabled { self.fm_index } else { 0.0 };
                                    let actual_lfo_amp = if self.lfo_enabled { 2.0 } else { 0.0 };
                                    let _ = self.tx.send(AudioCommand::UpdateFMIndex(actual_fm));
                                    let _ = self.tx.send(AudioCommand::UpdateLFOAmp(actual_lfo_amp));
                                } else {
                                    ui.add_enabled_ui(self.fm_enabled, |ui| {
                                        if scroll_slider(ui, &mut self.fm_index, 0.00..=20.0, "FM Index", false) {
                                            let _ = self.tx.send(AudioCommand::UpdateFMIndex(self.fm_index));
                                        }
                                    });
                                    ui.add_enabled_ui(self.lfo_enabled, |ui| {
                                        if scroll_slider(ui, &mut self.lfo_rate, 0.01..=2.0, "LFO Rate", false) {
                                            let _ = self.tx.send(AudioCommand::UpdateLFORate(self.lfo_rate));
                                        }
                                    });
                                }
                            });
                            
                            ui.add_space(5.0);

                            ui.group(|ui| {
                                ui.label(egui::RichText::new("State Variable Filter (LPF)").heading());
                                ui.separator();
                                let mut filter_changed = false;
                                filter_changed |= ui.checkbox(&mut self.filter_enabled, "Enable SVF Filter").changed();
                                
                                ui.add_enabled_ui(self.filter_enabled, |ui| {
                                    if scroll_slider(ui, &mut self.filter_cutoff, 20.0..=20000.0, "Cutoff", true) {
                                        filter_changed = true;
                                    }
                                    if scroll_slider(ui, &mut self.filter_resonance, 0.1..=10.0, "Resonance", false) {
                                        filter_changed = true;
                                    }
                                });
                                
                                if filter_changed {
                                    let c = if self.filter_enabled { self.filter_cutoff } else { 20000.0 };
                                    let r = if self.filter_enabled { self.filter_resonance } else { 0.707 };
                                    let _ = self.tx.send(AudioCommand::UpdateFilter(c, r));
                                }
                            });
                            
                            ui.add_space(5.0);

                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Envelope (ADSR)").heading());
                                ui.separator();
                                if scroll_slider(ui, &mut self.attack, 0.001..=2.0, "Attack", false) {
                                    let _ = self.tx.send(AudioCommand::UpdateAttack(self.attack));
                                }
                                if scroll_slider(ui, &mut self.decay, 0.001..=2.0, "Decay", false) {
                                    let _ = self.tx.send(AudioCommand::UpdateDecay(self.decay));
                                }
                                if scroll_slider(ui, &mut self.sustain, 0.0..=1.0, "Sustain", false) {
                                    let _ = self.tx.send(AudioCommand::UpdateSustain(self.sustain));
                                }
                                if scroll_slider(ui, &mut self.release, 0.001..=2.0, "Release", false) {
                                    let _ = self.tx.send(AudioCommand::UpdateRelease(self.release));
                                }
                            });
                            
                            ui.add_space(5.0);

                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Output").heading());
                                ui.separator();
                                if scroll_slider(ui, &mut self.volume, 0.0..=1.0, "Master Volume", false) {
                                    let _ = self.tx.send(AudioCommand::UpdateVolume(self.volume));
                                }
                            });
                        });
                    // Removed extra closing braces from the old layout
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                let total_height = ui.available_height();
                let section_height = (total_height - 60.0) / 3.0; // Subtract spacing
                
                ui.group(|ui| {
                    ui.label("ADSR Envelope");
                    Plot::new("adsr_plot")
                        .height(section_height)
                        .include_y(0.0)
                        .include_y(1.1)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .show(ui, |plot_ui| {
                            let points: PlotPoints = vec![
                                [0.0, 0.0],
                                [self.attack as f64, 1.0],
                                [(self.attack + self.decay) as f64, self.sustain as f64],
                                [(self.attack + self.decay + 0.5) as f64, self.sustain as f64], // Sustain hold
                                [(self.attack + self.decay + 0.5 + self.release) as f64, 0.0]
                            ].into();
                            plot_ui.line(Line::new(points).name("Envelope"));
                        });
                });

                ui.group(|ui| {
                    ui.label("Filter Response (LPF)");
                    Plot::new("filter_plot")
                        .height(section_height)
                        .x_axis_formatter(|v, _| {
                            if v.value > 0.0 { format!("{:.0}Hz", 10.0_f64.powf(v.value)) } 
                            else { "".to_string() }
                        })
                        .include_y(1.0)
                        .include_y(0.0)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .show(ui, |plot_ui| {
                            let cutoff = self.filter_cutoff as f64;
                            let res = self.filter_resonance as f64;
                            
                            let points: PlotPoints = (0..200).map(|i| {
                                // Logarithmic scale from 20Hz to 20kHz
                                let log_f = 1.3 + (i as f64 / 200.0) * 3.0; // 10^1.3 ~ 20Hz, 10^4.3 ~ 20kHz
                                let f = 10.0_f64.powf(log_f);
                                
                                // Simplified frequency response for visualization
                                let ratio = f / cutoff;
                                let denom = ((1.0 - ratio*ratio).powi(2) + (ratio/res).powi(2)).sqrt();
                                let mut mag = 1.0 / denom.max(0.001);
                                if !self.filter_enabled { mag = 1.0; }
                                [log_f, mag.min(1.5)]
                            }).collect();
                            plot_ui.line(Line::new(points).name("Magnitude"));
                        });
                });
                    
                ui.group(|ui| {
                    ui.label("Waveform Shape");
                    Plot::new("waveform_plot")
                        .height(ui.available_height() - 20.0) // Take remaining height
                        .include_x(0.0)
                        .include_x(std::f64::consts::TAU)
                        .include_y(1.0)
                        .include_y(-1.0)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .x_axis_formatter(|grid_mark, _range| {
                            let val = grid_mark.value;
                            if val.abs() < 0.001 { "0".to_string() }
                            else if (val - std::f64::consts::PI).abs() < 0.01 { "π".to_string() }
                            else if (val - std::f64::consts::TAU).abs() < 0.01 { "2π".to_string() }
                            else { format!("{:.1}", val) }
                        })
                        .show(ui, |plot_ui| {
                                let mut active_freqs: Vec<f64> = self.active_voices.iter().filter_map(|&f| f.map(|v| v as f64)).collect();
                                if active_freqs.is_empty() {
                                    active_freqs.push(self.frequency as f64);
                                }

                                // Show exactly one cycle (0 to 2Pi) of the fundamental/first frequency
                                let f_ref = active_freqs[0].max(20.0);
                                
                                let points: PlotPoints = (0..1000).map(|i| {
                                    let x_radians = (i as f64 / 1000.0) * std::f64::consts::TAU;
                                    let t = x_radians / (f_ref * std::f64::consts::TAU);
                                    
                                    let fm_idx = if self.fm_enabled { self.fm_index as f64 } else { 0.0 };
                                    let vol = self.volume as f64;
                                    
                                    let mut y_total = 0.0;
                                    for &f in &active_freqs {
                                        let mod_phase = t * f * fm_idx * std::f64::consts::TAU;
                                        let modulator = mod_phase.sin();
                                        
                                        let carrier_phase = t * f * std::f64::consts::TAU + (modulator * 200.0 / f.max(1.0));
                                        
                                        let mut phase = carrier_phase % std::f64::consts::TAU;
                                        if phase < 0.0 { phase += std::f64::consts::TAU; }
                                        let normalized_phase = phase / std::f64::consts::TAU;
                                        
                                        let y = match self.waveform as u8 {
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
                                        y_total += y;
                                    }
                                    
                                    // Normalize the multi-frequency sum to prevent visual clipping
                                    y_total /= active_freqs.len() as f64;
                                    y_total *= vol;
                                    
                                    [x_radians, y_total]
                                }).collect();
                                
                                plot_ui.line(Line::new(points).name("Waveform"));
                            });
                    });
            });
        });
    }
}
