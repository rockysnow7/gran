use crate::{effects::Effect, player::SAMPLE_RATE};

pub const SAMPLES_PER_GRAIN: usize = 512;

pub type Grain = [f32; SAMPLES_PER_GRAIN];

/// The data passed to an effect.
#[derive(Clone)]
pub struct EffectInput {
    pub grain: Grain,
    pub time_since_start_of_beat: f32, // in seconds
}

pub trait Sound: Send + Sync {
    fn next_sample(&mut self) -> f32;
    fn next_grain(&mut self) -> Grain;
    fn add_effect(&mut self, effect: Box<dyn Effect>);
    fn update_sample_rate(&mut self, sample_rate: usize);
    fn clone_box(&self) -> Box<dyn Sound>;
    fn secs_per_beat(&self) -> Option<f32>;
}

pub struct Composition {
    sounds: Vec<Box<dyn Sound>>,
    effects: Vec<Box<dyn Effect>>,
    secs_since_start: f32,
}

impl Composition {
    pub fn new(sounds: Vec<Box<dyn Sound>>, effects: Vec<Box<dyn Effect>>) -> Self {
        Self { sounds, effects, secs_since_start: 0.0 }
    }
}

impl Sound for Composition {
    fn secs_per_beat(&self) -> Option<f32> {
        None
    }

    fn add_effect(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }

    fn clone_box(&self) -> Box<dyn Sound> {
        Box::new(Self {
            sounds: self.sounds.iter().map(|s| s.clone_box()).collect(),
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
            secs_since_start: self.secs_since_start,
        })
    }

    fn update_sample_rate(&mut self, sample_rate: usize) {
        for sound in &mut self.sounds {
            sound.update_sample_rate(sample_rate);
        }
    }

    fn next_sample(&mut self) -> f32 {
        self.sounds.iter_mut().map(|sound| sound.next_sample()).sum()
    }

    fn next_grain(&mut self) -> Grain {
        let mut grain = [0.0; SAMPLES_PER_GRAIN];
        for sound in &mut self.sounds {
            let sound_grain = sound.next_grain();
            for (i, sample) in sound_grain.iter().enumerate() {
                grain[i] += sample;
            }
        }

        for effect in &mut self.effects {
            let input = EffectInput {
                grain,
                time_since_start_of_beat: self.secs_since_start,
            };
            let output = effect.apply(input);
            grain = output.grain;
        }

        self.secs_since_start += SAMPLES_PER_GRAIN as f32 / *SAMPLE_RATE as f32;

        grain
    }
}

pub struct CompositionBuilder {
    sounds: Vec<Box<dyn Sound>>,
    effects: Vec<Box<dyn Effect>>,
}

impl CompositionBuilder {
    pub fn new() -> Self {
        Self { sounds: Vec::new(), effects: Vec::new() }
    }

    pub fn sound(mut self, sound: Box<dyn Sound>) -> Self {
        self.sounds.push(sound);
        self
    }

    pub fn effect(mut self, effect: Box<dyn Effect>) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn build(self) -> Composition {
        Composition::new(self.sounds, self.effects)
    }
}
