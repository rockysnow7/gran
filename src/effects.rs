use crate::sound::{Grain, SoundInput, SAMPLES_PER_GRAIN};

pub trait Effect: Send + Sync {
    fn clone_box(&self) -> Box<dyn Effect>;
    fn apply(&self, input: SoundInput) -> Grain;
}

#[derive(Clone, Copy)]
/// Adjusts the volume of every grain.
pub struct Volume(pub f32);

impl Effect for Volume {
    fn apply(&self, input: SoundInput) -> Grain {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];
        for i in 0..SAMPLES_PER_GRAIN {
            new_grain[i] = input.grain[i] * self.0;
        }

        new_grain
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// A beat in a `Pattern`.
#[derive(Clone, Copy)]
pub enum PatternBeat {
    /// The grain should be played on this beat. Equivalent to `PlayWithVolume(1.0)`.
    Play,
    /// The grain should not be played on this beat. Equivalent to `PlayWithVolume(0.0)`.
    Skip,
    /// The grain should be played on this beat, with a volume multiplier.
    PlayWithVolume(f32),
}

/// Sequences a sound into a pattern of beats at given volumes.
#[derive(Clone)]
pub struct Pattern(pub Vec<PatternBeat>);

impl Effect for Pattern {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn apply(&self, input: SoundInput) -> Grain {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];
        for i in 0..SAMPLES_PER_GRAIN {
            new_grain[i] = match self.0[input.beat_number % self.0.len()] {
                PatternBeat::Play => input.grain[i],
                PatternBeat::Skip => 0.0,
                PatternBeat::PlayWithVolume(volume) => input.grain[i] * volume,
            };
        }

        new_grain
    }
}

impl Pattern {
    /// Randomly multiply the volume of each beat by a random value in the range `-range_multiplier..=range_multiplier`.
    pub fn humanize(self, range_multiplier: f32) -> Self {
        let mut new_pattern = self.0;
        for beat in new_pattern.iter_mut() {
            *beat = match beat {
                PatternBeat::Play => {
                    let volume = 1.0;
                    let range = volume * range_multiplier;
                    let random_offset = rand::random_range(-range..=range);

                    PatternBeat::PlayWithVolume(volume + random_offset)
                },
                PatternBeat::Skip => PatternBeat::Skip,
                PatternBeat::PlayWithVolume(volume) => {
                    let range = *volume * range_multiplier;
                    let random_offset = rand::random_range(-range..=range);

                    PatternBeat::PlayWithVolume(*volume + random_offset)
                },
            };
        }

        Self(new_pattern)
    }
}
