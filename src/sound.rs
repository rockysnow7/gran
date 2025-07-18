use std::collections::HashMap;

const SECS_PER_BEAT: f32 = 1.5;
pub const SAMPLES_PER_GRAIN: usize = 256;

pub type Grain = [f32; SAMPLES_PER_GRAIN];

pub trait Sound: Send + Sync {
    fn next_sample(&mut self) -> f32;
    fn next_grain(&mut self) -> Grain;
    fn add_effect(&mut self, effect: Box<dyn Effect>);
    fn update_sample_rate(&mut self, sample_rate: usize);
    fn clone_box(&self) -> Box<dyn Sound>;
}

pub struct Sample {
    samples: Vec<f32>,
    index: usize,
    effects: Vec<Box<dyn Effect>>,
}

impl Sample {
    pub fn new(samples: Vec<f32>, sample_rate: usize) -> Self {
        let mut samples = samples;
        let samples_per_beat = (sample_rate as f32 * SECS_PER_BEAT) as usize;
        samples.extend(vec![0.0; samples_per_beat - samples.len()]);

        Self { samples, index: 0, effects: Vec::new() }
    }
}

impl Sound for Sample {
    fn next_sample(&mut self) -> f32 {
        self.index = (self.index + 1) % self.samples.len();
        self.samples[self.index]
    }

    fn next_grain(&mut self) -> Grain {
        let mut grain = [0.0; SAMPLES_PER_GRAIN];
        for sample in &mut grain {
            *sample = self.next_sample();
        }

        for effect in &mut self.effects {
            grain = effect.apply(grain);
        }

        grain
    }

    fn update_sample_rate(&mut self, sample_rate: usize) {
        let samples_per_beat = (sample_rate as f32 * SECS_PER_BEAT) as usize;
        self.samples.extend(vec![0.0; samples_per_beat - self.samples.len()]);
    }

    fn clone_box(&self) -> Box<dyn Sound> {
        Box::new(Sample {
            samples: self.samples.clone(),
            index: self.index,
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
        })
    }

    fn add_effect(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }
}

pub struct Composition {
    pub sounds: HashMap<String, Box<dyn Sound>>,
}

impl Composition {
    pub fn new() -> Self {
        Self { sounds: HashMap::new() }
    }

    pub fn add_sound(&mut self, name: String, sound: Box<dyn Sound>) {
        self.sounds.insert(name, sound);
    }
}

pub trait Effect: Send + Sync {
    fn clone_box(&self) -> Box<dyn Effect>;
    fn apply(&self, grain: Grain) -> Grain;
}

pub struct Gain(pub f32);

impl Effect for Gain {
    fn apply(&self, grain: Grain) -> Grain {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];
        for i in 0..SAMPLES_PER_GRAIN {
            new_grain[i] = grain[i] * self.0;
        }

        new_grain
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Gain(self.0))
    }
}
