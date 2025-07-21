use std::sync::Arc;
use crate::{effects::Effect, player::SAMPLE_RATE, sounds::{Grain, Sound, EffectInput, SAMPLES_PER_GRAIN}};

pub struct Oscillator {
    function: Arc<dyn Fn(f32) -> f32 + Send + Sync>,
    index: usize,
    /// The length of a beat in seconds.
    beat_length: f32,
    effects: Vec<Box<dyn Effect>>,
}

impl Clone for Oscillator {
    fn clone(&self) -> Self {
        Self {
            function: Arc::clone(&self.function),
            index: self.index,
            beat_length: self.beat_length,
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
        }
    }
}

impl Sound for Oscillator {
    fn next_sample(&mut self) -> f32 {
        self.index += 1; // this is gross, should figure out something nicer
        let t = self.index as f32 / *SAMPLE_RATE as f32;

        (self.function)(t)
    }

    fn next_grain(&mut self) -> Grain {
        let mut grain = [0.0; SAMPLES_PER_GRAIN];
        for sample in &mut grain {
            *sample = self.next_sample();
        }

        for effect in &mut self.effects {
            let beat_number = self.index / (*SAMPLE_RATE as f32 * self.beat_length) as usize;
            let input = EffectInput {
                grain,
                beat_number,
            };
            grain = effect.apply(input);
        }

        grain
    }

    fn update_sample_rate(&mut self, _sample_rate: usize) {} // does not affect anything

    fn clone_box(&self) -> Box<dyn Sound> {
        Box::new(Self {
            function: Arc::clone(&self.function),
            index: self.index,
            beat_length: self.beat_length,
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
        })
    }

    fn add_effect(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }
}

pub struct OscillatorBuilder {
    pub function: Option<Arc<dyn Fn(f32) -> f32 + Send + Sync>>,
    pub beat_length: Option<f32>,
    pub effects: Vec<Box<dyn Effect>>,
}

impl OscillatorBuilder {
    pub fn new() -> Self {
        Self {
            function: None,
            beat_length: None,
            effects: Vec::new(),
        }
    }

    pub fn function(mut self, function: impl Fn(f32) -> f32 + Send + Sync + 'static) -> Self {
        self.function = Some(Arc::new(function));
        self
    }

    pub fn beat_length(mut self, beat_length: f32) -> Self {
        self.beat_length = Some(beat_length);
        self
    }

    pub fn effect(mut self, effect: Box<dyn Effect>) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn build(self) -> Oscillator {
        Oscillator {
            function: self.function.unwrap(),
            index: 0,
            beat_length: self.beat_length.unwrap(),
            effects: self.effects,
        }
    }
}

pub mod waves {
    use std::f32::consts::PI;

    pub fn sine(frequency: f32) -> impl Fn(f32) -> f32 + Send + Sync {
        move |t: f32| (2.0 * PI * frequency * t).sin()
    }
}

pub enum Number {
    Number(f32),
    Oscillator(Oscillator),
}

impl Clone for Number {
    fn clone(&self) -> Self {
        match self {
            Number::Number(n) => Number::Number(*n),
            Number::Oscillator(osc) => Number::Oscillator(osc.clone()),
        }
    }
}

impl Number {
    pub fn next_value(&mut self) -> f32 {
        match self {
            Number::Number(number) => *number,
            Number::Oscillator(oscillator) => oscillator.next_sample(),
        }
    }

    pub fn plus(self, rhs: f32) -> Self {
        match self {
            Number::Number(number) => Number::Number(number + rhs),
            Number::Oscillator(oscillator) => {
                let function = move |t: f32| (oscillator.function)(t) + rhs;

                Number::Oscillator(Oscillator {
                    function: Arc::new(function),
                    index: oscillator.index,
                    beat_length: oscillator.beat_length,
                    effects: oscillator.effects,
                })
            },
        }
    }

    pub fn mul(self, rhs: f32) -> Self {
        match self {
            Number::Number(number) => Number::Number(number * rhs),
            Number::Oscillator(oscillator) => {
                let function = move |t: f32| (oscillator.function)(t) * rhs;

                Number::Oscillator(Oscillator {
                    function: Arc::new(function),
                    index: oscillator.index,
                    beat_length: oscillator.beat_length,
                    effects: oscillator.effects,
                })
            },
        }
    }

    pub fn clamp(self, min: f32, max: f32) -> Self {
        match self {
            Number::Number(number) => Number::Number(number.clamp(min, max)),
            Number::Oscillator(oscillator) => {
                let function = move |t: f32| (oscillator.function)(t).clamp(min, max);

                Number::Oscillator(Oscillator {
                    function: Arc::new(function),
                    index: oscillator.index,
                    beat_length: oscillator.beat_length,
                    effects: oscillator.effects,
                })
            },
        }
    }
}
