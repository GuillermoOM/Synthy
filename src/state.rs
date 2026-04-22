/// Commands passed from UI thread to the Audio thread.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AudioCommand {
    UpdateFrequency(f32),
    UpdateVolume(f32),
    UpdateFMIndex(f32),
    NoteOn(u8, f32), // voice_idx, frequency
    NoteOff(u8),     // voice_idx
    UpdateAttack(f32),
    UpdateDecay(f32),
    UpdateSustain(f32),
    UpdateRelease(f32),
    UpdateWaveform(f32),
    UpdateLFORate(f32),
}
