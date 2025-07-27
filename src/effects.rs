use crate::{Number, player::SAMPLE_RATE, sound::{EffectInput, Grain, SAMPLES_PER_GRAIN}};
use std::f32::consts::PI;

#[derive(Debug)]
pub enum OscillatorChange {
    Frequency(f32),
}

pub struct EffectOutput {
    pub grain: Grain,
    pub oscillator_changes: Vec<OscillatorChange>,
}

pub trait Effect: Send + Sync {
    fn clone_box(&self) -> Box<dyn Effect>;
    fn apply(&mut self, input: EffectInput) -> EffectOutput;
}

/// Adjusts the volume of every grain.
#[derive(Clone)]
pub struct Volume(pub Number);

impl Effect for Volume {
    fn apply(&mut self, input: EffectInput) -> EffectOutput {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];
        for i in 0..SAMPLES_PER_GRAIN {
            new_grain[i] = input.grain[i] * self.0.next_value();
        }

        EffectOutput {
            grain: new_grain,
            oscillator_changes: Vec::new(),
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct OnePoleFilter {
    pub previous_output: f32,
}

impl OnePoleFilter {
    pub fn new() -> Self {
        Self { previous_output: 0.0 }
    }

    fn process_sample(&mut self, sample: f32, cutoff: f32) -> f32 {
        let output = cutoff * sample + (1.0 - cutoff) * self.previous_output;
        self.previous_output = output;

        let saturated = (output * 0.7).tanh() * 1.4;

        saturated
    }
}

/// A ladder filter with a variable number of poles. If you use four poles, you can have feedback.
#[derive(Clone)]
pub struct LowPassFilter {
    cutoff_frequency: Number,
    resonance: Number,
    poles: Vec<OnePoleFilter>,
}

impl LowPassFilter {
    pub fn new(cutoff_frequency: Number, resonance: Number, num_poles: usize) -> Self {
        let mut poles = Vec::new();
        for _ in 0..num_poles {
            poles.push(OnePoleFilter::new());
        }

        Self { cutoff_frequency, resonance, poles }
    }

    fn process_sample(&mut self, mut sample: f32) -> f32 {
        if self.poles.len() == 4 { // only do feedback for 4-pole filter, anything less can't be heard and anything more kills your ears
            let resonance = self.resonance.next_value();
            assert!(resonance >= 0.0 && resonance <= 1.0);
            let feedback = 5.5 * resonance * self.poles.last().unwrap().previous_output;
            sample -= feedback;
        }

        let cutoff_frequency = self.cutoff_frequency.next_value();
        let cutoff = 1.0 - (-2.0 * PI * cutoff_frequency / *SAMPLE_RATE as f32).exp();
        for pole in &mut self.poles {
            sample = pole.process_sample(sample, cutoff);
        }

        sample
    }
}

impl Effect for LowPassFilter {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn apply(&mut self, input: EffectInput) -> EffectOutput {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];

        for i in 0..SAMPLES_PER_GRAIN {
            new_grain[i] = self.process_sample(input.grain[i]);
        }

        EffectOutput {
            grain: new_grain,
            oscillator_changes: Vec::new(),
        }
    }
}

/// Applies a soft saturation to the grain.
#[derive(Clone)]
pub struct Saturation {
    target_drive: Number,
    actual_drive: f32,
    mix: Number,
    slew_rate: f32,
}

impl Saturation {
    pub fn new(drive: Number, mix: Number, slew_rate: f32) -> Self {
        let mut target_drive = drive.clone();

        Self {
            target_drive: target_drive.clone(),
            actual_drive: target_drive.next_value() / 3.0,
            mix,
            slew_rate,
        }
    }

    pub fn update_actual_drive(&mut self) {
        let target_drive = self.target_drive.next_value();
        let max_change = self.slew_rate / *SAMPLE_RATE as f32;
        let diff = target_drive - self.actual_drive;
        let change = diff.clamp(-max_change, max_change);
        self.actual_drive += change;
    }

    pub fn process_sample(&mut self, sample: f32) -> f32 {
        self.update_actual_drive();

        let drive = if sample >= 0.0 {
            self.actual_drive
        } else {
            self.actual_drive * 0.9
        };

        let scaled = sample * drive;
        let fd = scaled.tanh();
        let gain = 2.0 / (1.0 + drive).sqrt();
        let wet = fd * gain;

        let mix = self.mix.next_value();
        let new_sample = mix * wet + (1.0 - mix) * sample;

        new_sample
    }
}

impl Effect for Saturation {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn apply(&mut self, input: EffectInput) -> EffectOutput {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];

        for i in 0..SAMPLES_PER_GRAIN {
            let sample = input.grain[i];
            new_grain[i] = self.process_sample(sample);
        }

        EffectOutput {
            grain: new_grain,
            oscillator_changes: Vec::new(),
        }
    }
}

/// A tape delay effect for slapback, echo, etc.
pub struct TapeDelay {
    buffer: Vec<f32>,
    read_delay: f32, // in seconds
    extra_space: usize,
    mix: Number,
    feedback: Number,
    wow_oscillator: Number,
    flutter_oscillator: Number,
    low_pass_filter: LowPassFilter,
    saturation: Saturation,
}

impl Clone for TapeDelay {
    fn clone(&self) -> Self {
        let read_offset = (self.read_delay * *SAMPLE_RATE as f32) as usize;
        let mut new_buffer = Vec::with_capacity(read_offset + self.extra_space);

        for sample in &self.buffer {
            new_buffer.insert(0, *sample);
        }

        Self {
            buffer: new_buffer,
            read_delay: self.read_delay,
            extra_space: self.extra_space,
            mix: self.mix.clone(),
            feedback: self.feedback.clone(),
            wow_oscillator: self.wow_oscillator.clone(),
            flutter_oscillator: self.flutter_oscillator.clone(),
            low_pass_filter: self.low_pass_filter.clone(),
            saturation: self.saturation.clone(),
        }
    }
}

impl TapeDelay {
    pub fn light(delay: f32) -> Self {
        Self::new(
            delay,
            Number::number(0.1),
            Number::number(0.1),
            0.001,
            0.1,
            0.005,
            1.0,
        )
    }

    pub fn new(
        read_delay: f32,
        mix: Number,
        feedback: Number,
        wow_range_pct: f32,
        wow_speed: f32,
        flutter_range_pct: f32,
        flutter_speed: f32,
    ) -> Self {
        let wow_range = wow_range_pct * read_delay;
        let flutter_range = flutter_range_pct * read_delay;
        let extra_space = ((wow_range + flutter_range) * *SAMPLE_RATE as f32) as usize; // to allow for wow and flutter
        let read_offset = (read_delay * *SAMPLE_RATE as f32) as usize;
        let buffer = Vec::with_capacity(read_offset + extra_space);

        Self {
            buffer,
            read_delay,
            extra_space,
            mix,
            feedback,
            wow_oscillator: Number::sine_around(0.0, wow_range, wow_speed),
            flutter_oscillator: Number::sine_around(0.0, flutter_range, flutter_speed),
            low_pass_filter: LowPassFilter::new(Number::number(6000.0), Number::number(0.3), 1),
            saturation: Saturation::new(Number::number(2.0), Number::number(0.7), 0.5),
        }
    }

    fn push_sample_to_buffer(&mut self, sample: f32) {
        if self.buffer.len() >= self.buffer.capacity() - self.extra_space {
            self.buffer.remove(0);
        }

        self.buffer.push(sample);
    }

    fn read_sample_from_buffer(&mut self) -> f32 {
        let read_index = self.extra_space;
        let wow = self.wow_oscillator.next_value();
        let flutter = self.flutter_oscillator.next_value();
        // convert wow and flutter from seconds to samples
        let wow_samples = wow * *SAMPLE_RATE as f32;
        let flutter_samples = flutter * *SAMPLE_RATE as f32;
        let read_index = (read_index as f32 + wow_samples + flutter_samples) as usize;

        self.buffer[read_index]
    }

    fn process_sample(&mut self, sample: f32) -> f32 {
        let buffer_duration = self.buffer.len() as f32 / *SAMPLE_RATE as f32;
        let delay_sample = if buffer_duration < self.read_delay {
            0.0
        } else {
            self.read_sample_from_buffer()
        };

        let processed = self.saturation.process_sample(delay_sample);
        let processed = self.low_pass_filter.process_sample(processed);

        let feedback = self.feedback.next_value();
        assert!(feedback >= 0.0 && feedback <= 1.0);
        let to_buffer = sample + feedback * processed;
        self.push_sample_to_buffer(to_buffer);

        let mix = self.mix.next_value();
        assert!(mix >= 0.0 && mix <= 1.0);
        let mixed = mix * processed + (1.0 - mix) * sample;

        mixed
    }
}

impl Effect for TapeDelay {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn apply(&mut self, input: EffectInput) -> EffectOutput {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];

        for i in 0..SAMPLES_PER_GRAIN {
            new_grain[i] = self.process_sample(input.grain[i]);
        }

        EffectOutput {
            grain: new_grain,
            oscillator_changes: Vec::new(),
        }
    }
}
