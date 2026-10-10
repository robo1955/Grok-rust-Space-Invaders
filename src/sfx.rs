//! Square-wave cabinet sounds, synthesized into WAV bytes at startup.

use crate::game::Sfx;
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

pub struct Audio {
    shoot: Option<Sound>,
    steps: [Option<Sound>; 4],
    invader_die: Option<Sound>,
    player_die: Option<Sound>,
    ufo: Option<Sound>,
    ufo_die: Option<Sound>,
    wave: Option<Sound>,
    pub muted: bool,
}

impl Audio {
    pub async fn load() -> Self {
        let steps = [
            tone(164.0, 45, 0.35),
            tone(146.0, 45, 0.35),
            tone(130.0, 45, 0.35),
            tone(116.0, 45, 0.35),
        ];
        let shoot = slide(920.0, 180.0, 70, 0.4);
        let invader_die = noise(130, 0.55);
        let player_die = slide(440.0, 55.0, 560, 0.5);
        let ufo = ufo_loop();
        let ufo_die = slide(980.0, 140.0, 220, 0.5);
        let wave = arpeggio(&[523.0, 659.0, 784.0, 1046.0], 80, 0.35);

        Self {
            shoot: load_one(&shoot).await,
            steps: [
                load_one(&steps[0]).await,
                load_one(&steps[1]).await,
                load_one(&steps[2]).await,
                load_one(&steps[3]).await,
            ],
            invader_die: load_one(&invader_die).await,
            player_die: load_one(&player_die).await,
            ufo: load_one(&ufo).await,
            ufo_die: load_one(&ufo_die).await,
            wave: load_one(&wave).await,
            muted: false,
        }
    }

    pub fn play(&mut self, sfx: Sfx) {
        if self.muted && !matches!(sfx, Sfx::UfoStop) {
            return;
        }
        match sfx {
            Sfx::Shoot => once(&self.shoot, 0.4),
            Sfx::Step(note) => once(&self.steps[(note as usize) % 4], 0.32),
            Sfx::InvaderDie => once(&self.invader_die, 0.5),
            Sfx::PlayerDie => once(&self.player_die, 0.55),
            Sfx::UfoDie => once(&self.ufo_die, 0.5),
            Sfx::Wave => once(&self.wave, 0.4),
            Sfx::UfoLoop => self.start_ufo(),
            Sfx::UfoStop => self.stop_ufo(),
        }
    }

    pub fn toggle_mute(&mut self, ufo_alive: bool) {
        self.muted = !self.muted;
        if self.muted {
            self.stop_ufo();
        } else if ufo_alive {
            self.start_ufo();
        }
    }

    fn start_ufo(&mut self) {
        if self.muted {
            return;
        }
        if let Some(sound) = &self.ufo {
            stop_sound(sound);
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: 0.22,
                },
            );
        }
    }

    fn stop_ufo(&mut self) {
        if let Some(sound) = &self.ufo {
            stop_sound(sound);
        }
    }
}

async fn load_one(bytes: &[u8]) -> Option<Sound> {
    load_sound_from_bytes(bytes).await.ok()
}

pub fn seed_from_clock() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as u64)
        .unwrap_or(1)
}

fn once(sound: &Option<Sound>, volume: f32) {
    if let Some(sound) = sound {
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }
}

const SAMPLE_RATE: u32 = 22_050;

fn tone(freq: f32, ms: u32, volume: f32) -> Vec<u8> {
    encode_wav(&voice(freq, freq, ms, volume, 0.5), SAMPLE_RATE)
}

fn slide(from: f32, to: f32, ms: u32, volume: f32) -> Vec<u8> {
    encode_wav(&voice(from, to, ms, volume, 0.15), SAMPLE_RATE)
}

fn voice(from: f32, to: f32, ms: u32, volume: f32, duty: f32) -> Vec<i16> {
    let count = (SAMPLE_RATE as u64 * ms as u64 / 1000).max(1) as usize;
    let mut phase = 0.0f32;
    let mut samples = Vec::with_capacity(count);
    for index in 0..count {
        let t = index as f32 / count as f32;
        let freq = from + (to - from) * t;
        phase = (phase + freq / SAMPLE_RATE as f32) % 1.0;
        let env = (1.0 - t) * (1.0 - t * 0.35);
        let sample = if phase < duty { volume } else { -volume };
        samples.push((sample * env * 30_000.0) as i16);
    }
    samples
}

fn noise(ms: u32, volume: f32) -> Vec<u8> {
    let count = (SAMPLE_RATE as u64 * ms as u64 / 1000).max(1) as usize;
    let mut state = 0xA5A5_1357u32;
    let mut samples = Vec::with_capacity(count);
    for index in 0..count {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let white = ((state >> 16) as i32 - 32_768) as f32 / 32_768.0;
        let t = index as f32 / count as f32;
        let env = (1.0 - t) * (1.0 - t);
        samples.push((white * volume * env * 28_000.0) as i16);
    }
    encode_wav(&samples, SAMPLE_RATE)
}

fn ufo_loop() -> Vec<u8> {
    let count = SAMPLE_RATE as usize / 2;
    let mut phase = 0.0f32;
    let mut samples = Vec::with_capacity(count);
    for index in 0..count {
        let t = index as f32 / SAMPLE_RATE as f32;
        let freq = if (t * 8.0) as i32 % 2 == 0 { 560.0 } else { 430.0 };
        phase = (phase + freq / SAMPLE_RATE as f32) % 1.0;
        let sample = if phase < 0.25 { 0.55 } else { -0.55 };
        samples.push((sample * 24_000.0) as i16);
    }
    encode_wav(&samples, SAMPLE_RATE)
}

fn arpeggio(notes: &[f32], ms: u32, volume: f32) -> Vec<u8> {
    let mut samples = Vec::new();
    for note in notes {
        samples.extend(voice(*note, *note, ms, volume, 0.5));
    }
    encode_wav(&samples, SAMPLE_RATE)
}

fn encode_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_size = samples.len() * 2;
    let mut bytes = Vec::with_capacity(44 + data_size);
    bytes.extend(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size as u32).to_le_bytes());
    bytes.extend(b"WAVE");
    bytes.extend(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend_from_slice(&(data_size as u32).to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_mono_pcm() {
        let bytes = encode_wav(&[0, 1000, -1000], 22_050);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[20..22], 1u16.to_le_bytes());
        assert_eq!(bytes.len(), 44 + 6);
    }
}
