use crate::state::AudioCommand;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::Receiver;

pub fn setup_audio_stream(
    cmd_receiver: Receiver<AudioCommand>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("Failed to find default output device")?;
    let config = device.default_output_config()?;

    // Build the graph using our definition in dsp.rs
    let (mut synth, handles) = crate::dsp::build_synth_graph();
    synth.set_sample_rate(config.sample_rate().0 as f64);

    let channels = config.channels() as usize;

    let err_fn = |err| eprintln!("Audio stream error: {:?}", err);

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Handle incoming commands from the UI thread
            while let Ok(cmd) = cmd_receiver.try_recv() {
                match cmd {
                    AudioCommand::UpdateFrequency(f) => {
                        // In poly mode, freq knob updates all voices simultaneously
                        for v in &handles.voices {
                            v.freq.set_value(f);
                        }
                    }
                    AudioCommand::UpdateVolume(v) => handles.volume.set_value(v),
                    AudioCommand::UpdateFMIndex(i) => handles.fm_index.set_value(i),
                    AudioCommand::NoteOn(idx, f) => {
                        if (idx as usize) < handles.voices.len() {
                            let v = &handles.voices[idx as usize];
                            v.freq.set_value(f);
                            v.gate.set_value(1.0);
                        }
                    }
                    AudioCommand::NoteOff(idx) => {
                        if (idx as usize) < handles.voices.len() {
                            handles.voices[idx as usize].gate.set_value(0.0);
                        }
                    }
                    AudioCommand::UpdateAttack(a) => handles.attack.set_value(a),
                    AudioCommand::UpdateDecay(d) => handles.decay.set_value(d),
                    AudioCommand::UpdateSustain(s) => handles.sustain.set_value(s),
                    AudioCommand::UpdateRelease(r) => handles.release.set_value(r),
                    AudioCommand::UpdateWaveform(w) => handles.waveform_select.set_value(w),
                    AudioCommand::UpdateLFORate(r) => handles.lfo_rate.set_value(r),
                }
            }

            for frame in data.chunks_mut(channels) {
                let (l, r) = synth.get_stereo();
                for (i, sample) in frame.iter_mut().enumerate() {
                    *sample = if i == 0 { l as f32 } else { r as f32 };
                }
            }
        },
        err_fn,
        None,
    )?;

    stream.play()?;
    Ok(stream)
}
