use std::f32::consts::PI;
use crate::{oscillators::Number, player::SAMPLE_RATE, sounds::{EffectInput, Grain, SAMPLES_PER_GRAIN}};

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

// /// Applies an attack-decay-sustain-release envelope to the grain.
// /// As sounds have a fixed duration, the `sustain_duration` is the duration of the sustain phase.
// /// The remaining duration is used for the release.
// /// All durations are in seconds.
// #[derive(Clone)]
// pub struct ADSR {
//     pub attack_duration: f32, // in seconds
//     pub decay_duration: f32, // in seconds
//     pub sustain_amplitude_multiplier: f32,
//     pub sustain_duration: f32, // in seconds
// }

// impl ADSR {
//     pub fn new(
//         attack_duration: f32,
//         decay_duration: f32,
//         sustain_amplitude_multiplier: f32,
//         sustain_duration: f32,
//     ) -> Self {
//         Self {
//             attack_duration,
//             decay_duration,
//             sustain_amplitude_multiplier,
//             sustain_duration,
//         }
//     }
// }

// impl Effect for ADSR {
//     fn clone_box(&self) -> Box<dyn Effect> {
//         Box::new(self.clone())
//     }

//     fn apply(&mut self, input: EffectInput) -> EffectOutput {
//         assert!(self.attack_duration + self.decay_duration + self.sustain_duration <= input.secs_per_beat);

//         let sustain_start = self.attack_duration + self.decay_duration;
//         let release_start = sustain_start + self.sustain_duration;

//         let mut new_grain = [0.0; SAMPLES_PER_GRAIN];
//         if input.time_since_start_of_beat < self.attack_duration {
//             // attack
//             let attack_progress = input.time_since_start_of_beat / self.attack_duration;
//             for i in 0..SAMPLES_PER_GRAIN {
//                 new_grain[i] = input.grain[i] * attack_progress;
//             }
//         } else if input.time_since_start_of_beat < sustain_start {
//             // decay
//             let sustain_amplitude_multiplier_diff = 1.0 - self.sustain_amplitude_multiplier;
//             let decay_progress = (input.time_since_start_of_beat - self.attack_duration) / self.decay_duration;
//             let sustain_amplitude_multiplier = 1.0 - sustain_amplitude_multiplier_diff * decay_progress;
//             for i in 0..SAMPLES_PER_GRAIN {
//                 new_grain[i] = input.grain[i] * sustain_amplitude_multiplier;
//             }
//         } else if input.time_since_start_of_beat < release_start {
//             // sustain
//             for i in 0..SAMPLES_PER_GRAIN {
//                 new_grain[i] = input.grain[i] * self.sustain_amplitude_multiplier;
//             }
//         } else {
//             // release
//             let release_duration = input.secs_per_beat - release_start;
//             let release_progress = (input.time_since_start_of_beat - release_start) / release_duration;
//             let release_amplitude_multiplier = self.sustain_amplitude_multiplier * (1.0 - release_progress);
//             for i in 0..SAMPLES_PER_GRAIN {
//                 new_grain[i] = input.grain[i] * release_amplitude_multiplier;
//             }
//         }

//         EffectOutput {
//             grain: new_grain,
//             oscillator_changes: Vec::new(),
//         }
//     }
// }

/// A low-pass or high-pass filter.
#[derive(Clone)]
pub enum Filter {
    LowPass {
        cutoff_frequency: Number,
        resonance: Number,
        state_1: f32,
        state_2: f32,
        previous_cutoff: f32,
    },
    HighPass {
        cutoff_frequency: Number,
        resonance: Number,
        state_1: f32,
        state_2: f32,
        previous_cutoff: f32,
    },
}

impl Filter {
    pub fn low_pass(cutoff_frequency: Number, resonance: Number) -> Self {
        Self::LowPass {
            cutoff_frequency,
            resonance,
            state_1: 0.0,
            state_2: 0.0,
            previous_cutoff: 0.0,
        }
    }

    pub fn high_pass(cutoff_frequency: Number, resonance: Number) -> Self {
        Self::HighPass {
            cutoff_frequency,
            resonance,
            state_1: 0.0,
            state_2: 0.0,
            previous_cutoff: 0.0,
        }
    }

    pub fn process_sample(&mut self, sample: f32) -> f32 {
        match self {
            Self::LowPass { cutoff_frequency, resonance, state_1, state_2, previous_cutoff } => {
                let cutoff = cutoff_frequency.next_value();
                let resonance = resonance.next_value();
                assert!(resonance < 0.95); // to save your ears

                let cuttoff_smooth = 0.9 * *previous_cutoff + 0.1 * cutoff;
                *previous_cutoff = cuttoff_smooth;

                let omega = 2.0 * PI * cuttoff_smooth / *SAMPLE_RATE as f32;
                let g = omega.tan();
                let k = 2.0 - 2.0 * resonance;

                let v_1 = (sample - *state_2 - k * *state_1) / (1.0 + k * g + g * g);
                let v_2 = *state_1 + g * v_1;
                let v_3 = *state_2 + g * v_2;

                *state_1 = v_2;
                *state_2 = v_3;

                v_3
            },
            Self::HighPass { cutoff_frequency, resonance, state_1, state_2, previous_cutoff } => {
                let cutoff = cutoff_frequency.next_value();
                let resonance = resonance.next_value();
                assert!(resonance < 0.95); // to save your ears

                let cuttoff_smooth = 0.9 * *previous_cutoff + 0.1 * cutoff;
                *previous_cutoff = cuttoff_smooth;

                let omega = 2.0 * PI * cuttoff_smooth / *SAMPLE_RATE as f32;
                let g = omega.tan();
                let k = 2.0 - 2.0 * resonance;

                let v_1 = (sample - *state_2 - k * *state_1) / (1.0 + k * g + g * g);
                let v_2 = *state_1 + g * v_1;
                let v_3 = *state_2 + g * v_2;

                *state_1 = v_2;
                *state_2 = v_3;

                v_1
            },
        }
    }
}

impl Effect for Filter {
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
    low_pass_filter: Filter,
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
            low_pass_filter: Filter::low_pass(Number::number(6000.0), Number::number(0.3)),
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

// /// Offsets the time of the grain to only play before or after a certain number of seconds.
// #[derive(Clone)]
// pub enum TimeOffset {
//     WaitUntil(f32),
//     StopAfter(f32),
// }

// impl Effect for TimeOffset {
//     fn clone_box(&self) -> Box<dyn Effect> {
//         Box::new(self.clone())
//     }

//     fn apply(&mut self, input: EffectInput) -> EffectOutput {
//         let time = input.beat_number as f32 * input.secs_per_beat + input.time_since_start_of_beat;

//         match self {
//             Self::WaitUntil(start_time) => {
//                 if time < *start_time {
//                     EffectOutput {
//                         grain: [0.0; SAMPLES_PER_GRAIN],
//                         oscillator_changes: Vec::new(),
//                     }
//                 } else {
//                     EffectOutput {
//                         grain: input.grain,
//                         oscillator_changes: Vec::new(),
//                     }
//                 }
//             },
//             Self::StopAfter(stop_time) => {
//                 if time > *stop_time {
//                     EffectOutput {
//                         grain: [0.0; SAMPLES_PER_GRAIN],
//                         oscillator_changes: Vec::new(),
//                     }
//                 } else {
//                     EffectOutput {
//                         grain: input.grain,
//                         oscillator_changes: Vec::new(),
//                     }
//                 }
//             }
//         }
//     }
// }
