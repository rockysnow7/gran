use crate::{oscillators::Number, sounds::{EffectInput, Grain, SAMPLES_PER_GRAIN}};

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
