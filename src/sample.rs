use std::{f32::consts::PI, fs::File, io::BufReader};

use rodio::{Decoder, Source};

use crate::{effects::Effect, sounds::{EffectInput, Grain, Sound, SAMPLES_PER_GRAIN}};

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

#[derive(Clone)]
pub enum SampleInput {
    Trigger,
}

#[derive(Clone)]
pub struct SampleInputAtTime {
    pub input: SampleInput,
    pub time: f32,
}

pub struct Sample {
    samples: Vec<f32>,
    secs_per_beat: f32,
    index: usize,
    pub effects: Vec<Box<dyn Effect>>,
    secs_since_start: f32,
    inputs: Vec<SampleInputAtTime>,
    play: bool,
}

impl Sample {
    pub fn new(
        samples: Vec<f32>,
        sample_rate: usize,
        secs_per_beat: f32,
        inputs: Vec<SampleInputAtTime>,
    ) -> Self {
        let target_samples = (sample_rate as f32 * secs_per_beat) as usize;
        let samples = normalize_sample_length(samples, target_samples);

        Self {
            samples,
            secs_per_beat,
            index: 0,
            effects: Vec::new(),
            secs_since_start: 0.0,
            inputs,
            play: false,
        }
    }

    fn handle_input(&mut self, input: SampleInput) {
        match input {
            SampleInput::Trigger => {
                self.index = 0;
                self.play = true;
            }
        }
    }

    fn update_inputs(&mut self) {
        if let Some(input) = self.inputs.first() {
            if self.secs_since_start >= input.time {
                self.handle_input(input.input.clone());
                self.inputs.remove(0);
            }
        }
    }
}

impl Sound for Sample {
    fn secs_per_beat(&self) -> Option<f32> {
        Some(self.secs_per_beat)
    }

    fn next_sample(&mut self) -> f32 {
        if !self.play {
            return 0.0;
        }

        self.index += 1;
        if self.index >= self.samples.len() {
            self.play = false;
            return 0.0;
        }

        self.samples[self.index]
    }

    fn next_grain(&mut self) -> Grain {
        self.update_inputs();

        let mut grain = [0.0; SAMPLES_PER_GRAIN];
        for sample in &mut grain {
            *sample = self.next_sample();
        }

        let time_since_start_of_beat = self.index as f32 / self.samples.len() as f32;
        for effect in &mut self.effects {
            let input = EffectInput {
                grain,
                time_since_start_of_beat,
            };
            let output = effect.apply(input);
            grain = output.grain;
        }

        grain
    }

    fn update_sample_rate(&mut self, sample_rate: usize) {
        let target_samples = (sample_rate as f32 * self.secs_per_beat) as usize;
        self.samples = normalize_sample_length(std::mem::take(&mut self.samples), target_samples);
    }

    fn clone_box(&self) -> Box<dyn Sound> {
        Box::new(Sample {
            samples: self.samples.clone(),
            secs_per_beat: self.secs_per_beat,
            index: self.index,
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
            secs_since_start: self.secs_since_start,
            inputs: self.inputs.clone(),
            play: self.play,
        })
    }

    fn add_effect(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }
}

// returns (samples, sample rate)
fn load_sample_wav(path: &str) -> (Vec<f32>, usize) {
    let mut reader = hound::WavReader::open(path).unwrap();
    let sample_rate = reader.spec().sample_rate;
    let samples: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();

    (samples.iter().map(|s| *s as f32 / i32::MAX as f32).collect(), sample_rate as usize)
}

// returns (samples, sample rate)
fn load_sample_mp3(path: &str) -> (Vec<f32>, usize) {
    let file = File::open(path).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();
    let sample_rate = source.sample_rate();

    let samples: Vec<f32> = source
        .into_iter()
        .map(|sample| sample / i16::MAX as f32)
        .collect();
    
    (samples, sample_rate as usize)
}

pub struct SampleBuilder {
    samples: Option<Vec<f32>>,
    sample_rate: Option<usize>,
    secs_per_beat: Option<f32>,
    effects: Vec<Box<dyn Effect>>,
    inputs: Vec<SampleInputAtTime>,
}

impl SampleBuilder {
    pub fn new() -> Self {
        Self {
            samples: None,
            sample_rate: None,
            secs_per_beat: None,
            effects: Vec::new(),
            inputs: Vec::new(),
        }
    }

    pub fn samples(mut self, samples: Vec<f32>) -> Self {
        self.samples = Some(samples);
        self
    }

    pub fn with_sample_rate(mut self, sample_rate: usize) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn samples_from_file(mut self, path: &str) -> Self {
        let (samples, sample_rate) = if path.ends_with(".wav") {
            load_sample_wav(path)
        } else if path.ends_with(".mp3") {
            load_sample_mp3(path)
        } else {
            panic!("Unsupported file type: {}", path);
        };

        self.samples = Some(samples);
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn secs_per_beat(mut self, secs_per_beat: f32) -> Self {
        self.secs_per_beat = Some(secs_per_beat);
        self
    }

    pub fn effect(mut self, effect: Box<dyn Effect>) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn inputs(mut self, inputs: Vec<SampleInputAtTime>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn build(self) -> Sample {
        let samples = self.samples.unwrap();
        let sample_rate = self.sample_rate.unwrap();
        let secs_per_beat = self.secs_per_beat.unwrap();

        let mut sample = Sample::new(samples, sample_rate, secs_per_beat, self.inputs);
        for effect in self.effects {
            sample.add_effect(effect);
        }

        sample
    }
}
