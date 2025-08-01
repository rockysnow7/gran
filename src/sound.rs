use crate::{effects::{Effect, EffectTrait}, oscillator::Oscillator, player::SAMPLE_RATE, sample::Sample};

pub const SAMPLES_PER_GRAIN: usize = 512;

pub type Grain = [f32; SAMPLES_PER_GRAIN];

/// The data passed to an effect.
#[derive(Clone)]
pub struct EffectInput {
    pub grain: Grain,
    pub time_since_start_of_beat: f32, // in seconds
}

pub trait SoundTrait: Send + Sync {
    fn next_sample(&mut self) -> f32;
    fn next_grain(&mut self) -> Grain;
    fn add_effect(&mut self, effect: Effect);
    fn update_sample_rate(&mut self, sample_rate: usize);
    fn clone_box(&self) -> Box<dyn SoundTrait>;
    fn secs_per_beat(&self) -> Option<f32>;
}

#[derive(Clone)]
pub struct Composition {
    sounds: Vec<Sound>,
    effects: Vec<Effect>,
    secs_since_start: f32,
}

impl Composition {
    pub fn new(sounds: Vec<Sound>, effects: Vec<Effect>) -> Self {
        Self { sounds, effects, secs_since_start: 0.0 }
    }
}

impl SoundTrait for Composition {
    fn secs_per_beat(&self) -> Option<f32> {
        None
    }

    fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    fn clone_box(&self) -> Box<dyn SoundTrait> {
        Box::new(Self {
            // sounds: self.sounds.iter().map(|s| s.clone_box()).collect(),
            sounds: self.sounds.clone(),
            effects: self.effects.clone(),
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
    sounds: Vec<Sound>,
    effects: Vec<Effect>,
}

impl CompositionBuilder {
    pub fn new() -> Self {
        Self { sounds: Vec::new(), effects: Vec::new() }
    }

    pub fn sound(mut self, sound: Sound) -> Self {
        self.sounds.push(sound);
        self
    }

    pub fn effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn build(self) -> Composition {
        Composition::new(self.sounds, self.effects)
    }
}

#[derive(Clone)]
pub enum Sound {
    Oscillator(Oscillator),
    Sample(Sample),
    Composition(Composition),
}

impl SoundTrait for Sound {
    fn clone_box(&self) -> Box<dyn SoundTrait> {
        Box::new(self.clone())
    }

    fn next_sample(&mut self) -> f32 {
        match self {
            Sound::Oscillator(oscillator) => oscillator.next_sample(),
            Sound::Sample(sample) => sample.next_sample(),
            Sound::Composition(composition) => composition.next_sample(),
        }
    }

    fn next_grain(&mut self) -> Grain {
        match self {
            Sound::Oscillator(oscillator) => oscillator.next_grain(),
            Sound::Sample(sample) => sample.next_grain(),
            Sound::Composition(composition) => composition.next_grain(),
        }
    }

    fn secs_per_beat(&self) -> Option<f32> {
        match self {
            Sound::Oscillator(oscillator) => oscillator.secs_per_beat(),
            Sound::Sample(sample) => sample.secs_per_beat(),
            Sound::Composition(composition) => composition.secs_per_beat(),
        }
    }

    fn add_effect(&mut self, effect: Effect) {
        match self {
            Sound::Oscillator(oscillator) => oscillator.add_effect(effect),
            Sound::Sample(sample) => sample.add_effect(effect),
            Sound::Composition(composition) => composition.add_effect(effect),
        }
    }

    fn update_sample_rate(&mut self, sample_rate: usize) {
        match self {
            Sound::Oscillator(oscillator) => oscillator.update_sample_rate(sample_rate),
            Sound::Sample(sample) => sample.update_sample_rate(sample_rate),
            Sound::Composition(composition) => composition.update_sample_rate(sample_rate),
        }
    }
}
