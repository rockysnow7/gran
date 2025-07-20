use crate::sound::{Grain, SAMPLES_PER_GRAIN, SoundInput};

pub trait Effect: Send + Sync {
    fn clone_box(&self) -> Box<dyn Effect>;
    fn apply(&self, input: SoundInput) -> Grain;
}

/// Adjusts the volume of a grain.
pub struct Gain(pub f32);

impl Effect for Gain {
    fn apply(&self, input: SoundInput) -> Grain {
        let mut new_grain = [0.0; SAMPLES_PER_GRAIN];
        for i in 0..SAMPLES_PER_GRAIN {
            new_grain[i] = input.grain[i] * self.0;
        }

        new_grain
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Gain(self.0))
    }
}

/// Only plays the grain if the beat number is in a given set.
pub struct Pattern {
    /// The beats at which the grain should be played.
    pub trigger_beats: Vec<usize>,
    /// The length of the pattern in beats.
    pub length: usize,
}

impl Effect for Pattern {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Pattern {
            trigger_beats: self.trigger_beats.clone(),
            length: self.length,
        })
    }

    fn apply(&self, input: SoundInput) -> Grain {
        let beat_number = input.beat_number % self.length;
        if self.trigger_beats.contains(&beat_number) {
            input.grain
        } else {
            [0.0; SAMPLES_PER_GRAIN]
        }
    }
}
