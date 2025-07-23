use std::f32::consts::PI;
use crate::{oscillators::Number, player::SAMPLE_RATE, sounds::{EffectInput, Grain, SAMPLES_PER_GRAIN}};

pub trait Effect: Send + Sync {
    fn clone_box(&self) -> Box<dyn Effect>;
    fn apply(&mut self, input: EffectInput) -> Grain;
}

/// Adjusts the volume of every grain.
#[derive(Clone)]
pub struct Volume(pub Number);

impl Effect for Volume {
    fn apply(&mut self, input: EffectInput) -> Grain {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];
        for i in 0..SAMPLES_PER_GRAIN {
            new_grain[i] = input.grain[i] * self.0.next_value();
        }

        new_grain
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// A beat in a `Pattern`.
#[derive(Clone)]
pub enum PatternBeat {
    /// The grain should be played on this beat. Equivalent to `PlayWithVolume(1.0)`.
    Play,
    /// The grain should not be played on this beat. Equivalent to `PlayWithVolume(0.0)`.
    Skip,
    /// The grain should be played on this beat, with a volume multiplier.
    PlayWithVolume(Number),
}

/// Sequences a sound into a pattern of beats at given volumes.
#[derive(Clone)]
pub struct Pattern(pub Vec<PatternBeat>);

impl Effect for Pattern {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn apply(&mut self, input: EffectInput) -> Grain {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];
        for i in 0..SAMPLES_PER_GRAIN {
            let pattern_len = self.0.len();
            new_grain[i] = match &mut self.0[input.beat_number % pattern_len] {
                PatternBeat::Play => input.grain[i],
                PatternBeat::Skip => 0.0,
                PatternBeat::PlayWithVolume(volume) => input.grain[i] * volume.next_value(),
            };
        }

        new_grain
    }
}

impl Pattern {
    /// Multiply the volume of each beat by a random value in the range `-range_multiplier..=range_multiplier`.
    pub fn humanize(self, range_multiplier: f32) -> Self {
        let mut new_pattern = self.0;
        for beat in new_pattern.iter_mut() {
            *beat = match beat {
                PatternBeat::Play => {
                    let volume = 1.0;
                    let range = volume * range_multiplier;
                    let random_offset = rand::random_range(-range..=range);

                    PatternBeat::PlayWithVolume(Number::number(volume + random_offset))
                },
                PatternBeat::Skip => PatternBeat::Skip,
                PatternBeat::PlayWithVolume(volume) => {
                    let range = volume.next_value() * range_multiplier;
                    let random_offset = rand::random_range(-range..=range);

                    PatternBeat::PlayWithVolume(volume.clone().plus_f32(random_offset))
                },
            };
        }

        Self(new_pattern)
    }
}

#[derive(Clone)]
pub struct ADSR {
    pub attack_duration: f32,
    pub decay_duration: f32,
    pub sustain_amplitude_multiplier: f32,
    pub sustain_duration: f32,
}

impl ADSR {
    pub fn new(
        attack_duration: f32,
        decay_duration: f32,
        sustain_amplitude_multiplier: f32,
        sustain_duration: f32,
    ) -> Self {
        Self {
            attack_duration,
            decay_duration,
            sustain_amplitude_multiplier,
            sustain_duration,
        }
    }
}

impl Effect for ADSR {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn apply(&mut self, input: EffectInput) -> Grain {
        assert!(self.attack_duration + self.decay_duration + self.sustain_duration < input.secs_per_beat);

        let sustain_start = self.attack_duration + self.decay_duration;
        let release_start = sustain_start + self.sustain_duration;

        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];
        if input.time_since_start_of_beat < self.attack_duration {
            // attack
            let attack_progress = input.time_since_start_of_beat / self.attack_duration;
            for i in 0..SAMPLES_PER_GRAIN {
                new_grain[i] = input.grain[i] * attack_progress;
            }
        } else if input.time_since_start_of_beat < sustain_start {
            // decay
            let sustain_amplitude_multiplier_diff = 1.0 - self.sustain_amplitude_multiplier;
            let decay_progress = (input.time_since_start_of_beat - self.attack_duration) / self.decay_duration;
            let sustain_amplitude_multiplier = 1.0 - sustain_amplitude_multiplier_diff * decay_progress;
            for i in 0..SAMPLES_PER_GRAIN {
                new_grain[i] = input.grain[i] * sustain_amplitude_multiplier;
            }
        } else if input.time_since_start_of_beat < release_start {
            // sustain
            for i in 0..SAMPLES_PER_GRAIN {
                new_grain[i] = input.grain[i] * self.sustain_amplitude_multiplier;
            }
        } else {
            // release
            let release_duration = input.secs_per_beat - release_start;
            let release_progress = (input.time_since_start_of_beat - release_start) / release_duration;
            let release_amplitude_multiplier = self.sustain_amplitude_multiplier * (1.0 - release_progress);
            for i in 0..SAMPLES_PER_GRAIN {
                new_grain[i] = input.grain[i] * release_amplitude_multiplier;
            }
        }

        new_grain
    }
}

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

    fn apply(&mut self, input: EffectInput) -> Grain {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];

        for i in 0..SAMPLES_PER_GRAIN {
            new_grain[i] = self.process_sample(input.grain[i]);
        }

        new_grain
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
            actual_drive: target_drive.next_value() / 2.0,
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
}

impl Effect for Saturation {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn apply(&mut self, input: EffectInput) -> Grain {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];

        for i in 0..SAMPLES_PER_GRAIN {
            let sample = input.grain[i];

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
            new_grain[i] = new_sample;
        }

        new_grain
    }
}
