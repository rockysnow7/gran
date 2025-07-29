use std::f32::consts::PI;
use crate::player::SAMPLE_RATE;

#[derive(Clone, Debug)]
pub struct LFO {
    wave_function: Box<WaveFunction>,
    phase: f32,
}

impl LFO {
    pub fn next_value(&mut self) -> f32 {
        let dt = 1.0 / *SAMPLE_RATE as f32;
        self.wave_function.next_value(&mut self.phase, dt)
    }
}

pub struct LFOBuilder {
    wave_function: Option<WaveFunction>,
    phase: f32,
}

impl LFOBuilder {
    pub fn new() -> Self {
        LFOBuilder {
            wave_function: None,
            phase: 0.0,
        }
    }

    pub fn wave_function(mut self, wave_function: WaveFunction) -> Self {
        self.wave_function = Some(wave_function);
        self
    }

    pub fn phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }

    pub fn build(self) -> LFO {
        LFO {
            wave_function: Box::new(self.wave_function.unwrap()),
            phase: self.phase,
        }
    }
}

#[derive(Debug)]
pub enum Number {
    Number {
        value: f32,
        plus: f32,
        mul: f32,
    },
    Oscillator {
        oscillator: LFO,
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

    pub fn oscillator(oscillator: LFO) -> Self {
        Number::Oscillator { oscillator, plus: 0.0, mul: 1.0 }
    }

    /// Create a sine wave that oscillates around a middle value with a given frequency.
    pub fn sine_around(middle: f32, plus_or_minus: f32, frequency: f32) -> Self {
        let oscillator = LFOBuilder::new()
            .wave_function(WaveFunction::Sine {
                frequency: Number::number(frequency),
                amplitude: Number::number(plus_or_minus),
                phase: Number::number(0.0),
            })
            .build();

        Number::oscillator(oscillator).plus_f32(middle)
    }

    /// Create a square wave that oscillates around a middle value with a given frequency.
    pub fn square_around(middle: f32, plus_or_minus: f32, frequency: f32) -> Self {
        let oscillator = LFOBuilder::new()
            .wave_function(WaveFunction::Square {
                frequency: Number::number(frequency),
                amplitude: Number::number(plus_or_minus),
                phase: Number::number(0.0),
            })
            .build();

        Number::oscillator(oscillator).plus_f32(middle)
    }

    pub fn next_value(&mut self) -> f32 {
        match self {
            Number::Number { value, plus, mul } => *mul * *value + *plus,
            Number::Oscillator { oscillator, plus, mul } => {
                let value = oscillator.next_value();

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

#[derive(Clone, Debug)]
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
    WhiteNoise {
        amplitude: Number,
    },
    PinkNoise {
        amplitude: Number,
        generators: Vec<f32>,
        call_count: usize,
    },
}

fn poly_blep(phase: f32, phase_increment: f32) -> f32 {
    if phase < phase_increment {
        let t = phase / phase_increment;

        -t * t + 2.0 * t - 1.0 // -t^2 + 2t - 1
    } else if phase > 1.0 - phase_increment {
        let t = (phase - 1.0) / phase_increment;

        t * t + 2.0 * t + 1.0 // t^2 + 2t + 1
    } else {
        0.0
    }
}

impl WaveFunction {
    pub fn white_noise(amplitude: Number) -> Self {
        Self::WhiteNoise { amplitude }
    }

    pub fn pink_noise(amplitude: Number, num_generators: usize) -> Self {
        let generators = vec![0.0; num_generators];

        Self::PinkNoise { amplitude, generators, call_count: 0 }
    }

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
                
                *accumulated_phase += 2.0 * PI * freq * dt;
                *accumulated_phase = *accumulated_phase % (2.0 * PI);

                let phase_offset = phase.next_value();
                let normalized_phase = (*accumulated_phase + phase_offset) / (2.0 * PI);
                let normalized_phase = normalized_phase - normalized_phase.floor();

                let mut square = if normalized_phase < 0.5 { 1.0 } else { -1.0 };

                // smooth the rising edge
                let phase_increment = freq / *SAMPLE_RATE as f32;
                square += poly_blep(normalized_phase, phase_increment);
                
                // smooth the falling edge
                let shifted_phase = (normalized_phase + 0.5) % 1.0;
                square -= poly_blep(shifted_phase, phase_increment);

                let amp = amplitude.next_value();
                amp * square
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

                let mut sawtooth = 2.0 * normalized_phase - 1.0;

                let phase_increment = freq / *SAMPLE_RATE as f32;
                sawtooth -= poly_blep(normalized_phase, phase_increment);

                amp * sawtooth
            },
            WaveFunction::WhiteNoise { amplitude } => {
                let amp = amplitude.next_value();
                let noise = rand::random_range(-1.0..=1.0);

                amp * noise
            },
            WaveFunction::PinkNoise { amplitude, generators, call_count } => {
                // voss-mccartney
                let amp = amplitude.next_value();

                if *call_count >= 2usize.pow(generators.len() as u32) {
                    *call_count = 0;
                }

                // update the generators
                for i in 0..generators.len() {
                    if *call_count % 2usize.pow(i as u32) == 0 {
                        generators[i] = rand::random_range(-1.0..=1.0);
                    }
                }

                let scale_factor = 1.0 / 3.0f32.sqrt();
                let noise = generators.iter().sum::<f32>() * scale_factor;

                *call_count += 1;

                amp * noise
            },
        }
    }
}
