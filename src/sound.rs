use std::{collections::HashMap, f32::consts::PI};

pub const SAMPLES_PER_GRAIN: usize = 512;

/// Returns a Hanning window of the given size.
fn hanning_window(grain_size: usize) -> Vec<f32> {
    (0..grain_size)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / (grain_size as f32 - 1.0)).cos()))
        .collect()
}

/// Merges a grain into a buffer, with the given overlap percentage. The grain is assumed to be windowed already.
fn merge_grain_into_buffer(buffer: &[f32], grain: &[f32], overlap: f32) -> Vec<f32> {
    let overlap_len = (overlap * SAMPLES_PER_GRAIN as f32) as usize;
    let buffer_keep_len = buffer.len() - overlap_len;
    let buffer_keep = &buffer[..buffer_keep_len];
    let buffer_overlap = &buffer[buffer_keep_len..];
    let grain_overlap = &grain[..overlap_len];
    let grain_keep = &grain[overlap_len..];

    let overlap: Vec<_> = buffer_overlap.iter().zip(grain_overlap.iter()).map(|(a, b)| a + b).collect();
    let concat = [buffer_keep, &overlap, grain_keep].concat();

    concat
}

/// Compresses a sample by the given speed factor. The speed must be between 0.0 and 1.0 (inclusive).
fn compress(samples: &[f32], speed: f32) -> Vec<f32> {
    assert!(speed > 0.0 && speed <= 1.0);

    let grains: Vec<_> = samples.chunks(SAMPLES_PER_GRAIN).collect();
    let standard_window = hanning_window(SAMPLES_PER_GRAIN);
    let last_grain_len = grains.last().unwrap().len();
    let final_window = hanning_window(last_grain_len);

    let mut grains: Vec<Vec<_>> = grains[..grains.len() - 1]
        .iter()
        .map(|grain| grain
            .iter()
            .zip(standard_window.iter())
            .map(|(a, b)| a * b)
            .collect())
        .collect();
    let mut final_grain_windowed: Vec<_> = grains
        .last()
        .unwrap()
        .iter()
        .zip(final_window.iter())
        .map(|(a, b)| a * b)
        .collect();
    let final_grain_padding = SAMPLES_PER_GRAIN - last_grain_len;
    final_grain_windowed.extend(vec![0.0; final_grain_padding]);
    grains.push(final_grain_windowed);

    let mut buffer = grains.first().unwrap().clone();
    for grain in grains.iter().skip(1) {
        buffer = merge_grain_into_buffer(&buffer, grain, speed);
    }
    buffer.truncate(buffer.len() - final_grain_padding);

    buffer
}

fn normalize_sample_length(samples: Vec<f32>, target_length: usize) -> Vec<f32> {
    if samples.len() == target_length {
        samples
    } else if samples.len() < target_length {
        // pad with silence
        let mut result = samples;
        result.extend(vec![0.0; target_length - result.len()]);
        result
    } else {
        // resample to exact target length
        let speed = target_length as f32 / samples.len() as f32;
        let compressed = compress(&samples, speed);

        if compressed.len() > target_length {
            compressed[0..target_length].to_vec()
        } else if compressed.len() < target_length {
            let mut compressed = compressed;
            compressed.extend(vec![0.0; target_length - compressed.len()]);
            compressed
        } else {
            compressed
        }
    }
}

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
    secs_per_beat: f32,
    index: usize,
    effects: Vec<Box<dyn Effect>>,
}

impl Sample {
    pub fn new(
        samples: Vec<f32>,
        sample_rate: usize,
        secs_per_beat: f32,
    ) -> Self {
        let target_samples = (sample_rate as f32 * secs_per_beat) as usize;
        let samples = normalize_sample_length(samples, target_samples);

        Self {
            samples,
            secs_per_beat,
            index: 0,
            effects: Vec::new(),
        }
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
        let target_samples = (sample_rate as f32 * self.secs_per_beat) as usize;
        self.samples = normalize_sample_length(std::mem::take(&mut self.samples), target_samples);
        // println!("updated sample_rate: {:?}", sample_rate);
    }

    fn clone_box(&self) -> Box<dyn Sound> {
        Box::new(Sample {
            samples: self.samples.clone(),
            secs_per_beat: self.secs_per_beat,
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
