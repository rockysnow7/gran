use std::f32::consts::PI;
use crate::{effects::Effect, player::SAMPLE_RATE, sounds::{EffectInput, Grain, Sound, SAMPLES_PER_GRAIN}};

pub struct Oscillator {
    wave_function: Box<WaveFunction>,
    index: usize,
    /// The length of a beat in seconds.
    beat_length: f32,
    beat_number: usize,
    effects: Vec<Box<dyn Effect>>,
    phase: f32,
}

impl Clone for Oscillator {
    fn clone(&self) -> Self {
        Self {
            wave_function: self.wave_function.clone(),
            index: self.index,
            beat_length: self.beat_length,
            beat_number: self.beat_number,
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
            phase: self.phase,
        }
    }
}

impl Sound for Oscillator {
    fn secs_per_beat(&self) -> f32 {
        self.beat_length
    }

    fn next_sample(&mut self) -> f32 {
        let samples_per_beat = *SAMPLE_RATE as f32 * self.beat_length;
        self.index += 1;
        if self.index >= samples_per_beat as usize {
            self.beat_number += 1;
            self.index = 0;
        }

        let dt = 1.0 / *SAMPLE_RATE as f32;

        self.wave_function.next_value(&mut self.phase, dt)
    }

    fn next_grain(&mut self) -> Grain {
        let mut grain = [0.0; SAMPLES_PER_GRAIN];
        for sample in &mut grain {
            *sample = self.next_sample();
        }

        for effect in &mut self.effects {
            let samples_per_beat = *SAMPLE_RATE as f32 * self.beat_length;
            let time_since_start_of_beat = self.index as f32 / samples_per_beat;
            let input = EffectInput {
                grain,
                beat_number: self.beat_number,
                time_since_start_of_beat,
                secs_per_beat: self.beat_length,
            };
            grain = effect.apply(input);
        }

        grain
    }

    fn update_sample_rate(&mut self, _sample_rate: usize) {} // does not affect anything

    fn clone_box(&self) -> Box<dyn Sound> {
        Box::new(Self {
            wave_function: self.wave_function.clone(),
            index: self.index,
            beat_length: self.beat_length,
            beat_number: self.beat_number,
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
            phase: self.phase,
        })
    }

    fn add_effect(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }
}

pub struct OscillatorBuilder {
    pub wave_function: Option<WaveFunction>,
    pub beat_length: Option<f32>,
    pub effects: Vec<Box<dyn Effect>>,
}

impl OscillatorBuilder {
    pub fn new() -> Self {
        Self {
            wave_function: None,
            beat_length: None,
            effects: Vec::new(),
        }
    }

    pub fn wave_function(mut self, wave_function: WaveFunction) -> Self {
        self.wave_function = Some(wave_function);
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
            wave_function: Box::new(self.wave_function.unwrap()),
            index: 0,
            beat_length: self.beat_length.unwrap(),
            beat_number: 0,
            effects: self.effects,
            phase: 0.0,
        }
    }
}

#[derive(Clone)]
pub enum WaveFunction {
    Sine {
        frequency: Number,
        amplitude: Number,
        phase: Number,
    },
    Square {
        frequency: Number,
        amplitude: Number,
        phase: Number,
    },
    Triangle {
        frequency: Number,
        amplitude: Number,
        phase: Number,
    },
    Sawtooth {
        frequency: Number,
        amplitude: Number,
        phase: Number,
    },
}

impl WaveFunction {
    pub fn next_value(&mut self, accumulated_phase: &mut f32, dt: f32) -> f32 {
        match self {
            WaveFunction::Sine { frequency, amplitude, phase } => {
                let freq = frequency.next_value();
                let amp = amplitude.next_value();
                let phase_offset = phase.next_value();

                *accumulated_phase += 2.0 * PI * freq * dt;
                *accumulated_phase = *accumulated_phase % (2.0 * PI);
                
                amp * (*accumulated_phase + phase_offset).sin()
            },
            WaveFunction::Square { frequency, amplitude, phase } => {
                let freq = frequency.next_value();
                let amp = amplitude.next_value();
                let phase_offset = phase.next_value();

                *accumulated_phase += 2.0 * PI * freq * dt;
                *accumulated_phase = *accumulated_phase % (2.0 * PI);

                let sin = (*accumulated_phase + phase_offset).sin();
                let sign = if sin > 0.0 { 1.0 } else { -1.0 };

                amp * sign
            },
            WaveFunction::Triangle { frequency, amplitude, phase } => {
                let freq = frequency.next_value();
                let amp = amplitude.next_value();
                let phase_offset = phase.next_value();

                *accumulated_phase += 2.0 * PI * freq * dt;
                *accumulated_phase = *accumulated_phase % (2.0 * PI);

                // normaalise phase from radians to [0, 1]
                let normalized_phase = (*accumulated_phase + phase_offset) / (2.0 * PI);
                let normalized_phase = normalized_phase - normalized_phase.floor();

                let triangle = if normalized_phase < 0.5 {
                    4.0 * normalized_phase - 1.0  // -1 to 1 for first half
                } else {
                    3.0 - 4.0 * normalized_phase   // 1 to -1 for second half
                };

                amp * triangle
            },
            WaveFunction::Sawtooth { frequency, amplitude, phase } => {
                let freq = frequency.next_value();
                let amp = amplitude.next_value();
                let phase_offset = phase.next_value();

                *accumulated_phase += 2.0 * PI * freq * dt;
                *accumulated_phase = *accumulated_phase % (2.0 * PI);

                // normaalise phase from radians to [0, 1]
                let normalized_phase = (*accumulated_phase + phase_offset) / (2.0 * PI);
                let normalized_phase = normalized_phase - normalized_phase.floor();

                let sawtooth = 2.0 * normalized_phase - 1.0;

                amp * sawtooth
            },
        }
    }
}

pub enum Number {
    Number {
        value: f32,
        plus: f32,
        mul: f32,
    },
    Oscillator {
        oscillator: Oscillator,
        plus: f32,
        mul: f32,
    },
}

impl Clone for Number {
    fn clone(&self) -> Self {
        match self {
            Number::Number { value, plus, mul } => Number::Number {
                value: value.clone(),
                plus: *plus,
                mul: *mul,
            },
            Number::Oscillator { oscillator, plus, mul } => Number::Oscillator {
                oscillator: oscillator.clone(),
                plus: *plus,
                mul: *mul,
            },
        }
    }
}

impl Number {
    pub fn number(value: f32) -> Self {
        Number::Number { value, plus: 0.0, mul: 1.0 }
    }

    pub fn oscillator(oscillator: Oscillator) -> Self {
        Number::Oscillator { oscillator, plus: 0.0, mul: 1.0 }
    }

    /// Create a sine wave that oscillates around a middle value with a given frequency.
    pub fn sine_around(middle: f32, plus_or_minus: f32, frequency: f32) -> Self {
        let oscillator = OscillatorBuilder::new()
            .wave_function(WaveFunction::Sine {
                frequency: Number::number(frequency),
                amplitude: Number::number(plus_or_minus),
                phase: Number::number(0.0),
            })
            .beat_length(0.0)
            .build();

        Number::oscillator(oscillator).plus_f32(middle)
    }

    pub fn next_value(&mut self) -> f32 {
        match self {
            Number::Number { value, plus, mul } => *mul * *value + *plus,
            Number::Oscillator { oscillator, plus, mul } => {
                let value = oscillator.next_sample();
                *mul * value + *plus
            },
        }
    }

    pub fn plus_f32(self, rhs: f32) -> Self {
        match self {
            Number::Number { value, plus, mul } => Number::Number {
                value: value.clone(),
                plus: plus + rhs,
                mul: mul.clone(),
            },
            Number::Oscillator { oscillator, plus, mul } => Number::Oscillator {
                oscillator: oscillator.clone(),
                plus: plus + rhs,
                mul: mul.clone(),
            },
        }
    }

    pub fn mul_f32(self, rhs: f32) -> Self {
        match self {
            Number::Number { value, plus, mul } => Number::Number {
                value: value.clone(),
                plus: plus,
                mul: mul * rhs,
            },
            Number::Oscillator { oscillator, plus, mul } => Number::Oscillator {
                oscillator: oscillator.clone(),
                plus: plus,
                mul: mul * rhs,
            },
        }
    }
}
