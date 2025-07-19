pub mod sound;
pub mod player;

pub use sound::{Grain, SAMPLES_PER_GRAIN, Sound, Sample, Composition, Effect, Gain};
pub use player::{play_composition, get_sample_rate};
