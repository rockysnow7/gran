use std::f32::consts::PI;
use crate::{effects::{Effect, OscillatorChange}, player::SAMPLE_RATE, sounds::{EffectInput, Grain, Sound, SAMPLES_PER_GRAIN}};

/// Convert a note name to a frequency in Hz.
/// `note_name` is a string like "A4", "C#3", etc.
/// The octave must be given. Only sharp notes are supported, not flats.
pub fn note(note_name: &str) -> f32 {
    let octave = note_name.chars().last().unwrap().to_digit(10).unwrap() as isize;
    let note_name = note_name.chars().take(note_name.len() - 1).collect::<String>();

    let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let note_index = notes.iter().position(|note| *note == note_name).unwrap() as isize;
    let diff_from_a_within_octave = note_index - 9;
    let diff_from_a_octaves = octave - 4;
    let diff_semitones = diff_from_a_within_octave + diff_from_a_octaves * 12;

    let freq = 440.0 * 2.0f32.powf(diff_semitones as f32 / 12.0);

    freq
}

/// An input to an oscillator. Like a simplified form of MIDI.
#[derive(Clone)]
pub enum OscillatorInput {
    Press(f32), // frequency in Hz
    PressSame, // press the same frequency as the last input
    Release,
}

/// An input to be sent to an oscillator at a given time.
#[derive(Clone)]
pub struct OscillatorInputAtTime {
    pub input: OscillatorInput,
    pub time: f32, // in seconds since the start of the oscillator
}

/// Attack-decay-sustain-release envelope settings for an oscillator.
#[derive(Clone)]
pub struct ADSR {
    pub attack_duration: f32, // in seconds
    pub decay_duration: f32, // in seconds
    pub sustain_amplitude_multiplier: f32,
    pub release_duration: f32, // in seconds
}

impl ADSR {
    pub fn new(attack_duration: f32, decay_duration: f32, sustain_amplitude_multiplier: f32, release_duration: f32) -> Self {
        Self { attack_duration, decay_duration, sustain_amplitude_multiplier, release_duration }
    }
}

pub struct Oscillator {
    wave_function: Box<WaveFunction>,
    index: usize,
    effects: Vec<Box<dyn Effect>>,
    phase: f32,
    last_input: Option<OscillatorInput>,
    index_at_last_input: Option<usize>,
    inputs: Vec<OscillatorInputAtTime>,
    secs_since_start: f32,
    secs_since_release: Option<f32>,
    adsr: Option<ADSR>,
}

impl Oscillator {
    fn apply_change(&mut self, change: OscillatorChange) {
        match change {
            OscillatorChange::Frequency(freq) => {
                match self.wave_function.as_mut() {
                    WaveFunction::Sine { frequency, .. } => *frequency = Number::number(freq),
                    WaveFunction::Square { frequency, .. } => *frequency = Number::number(freq),
                    WaveFunction::Triangle { frequency, .. } => *frequency = Number::number(freq),
                    WaveFunction::Sawtooth { frequency, .. } => *frequency = Number::number(freq),
                    WaveFunction::WhiteNoise { .. } | WaveFunction::PinkNoise { .. } => {},
                }
            },
        }
    }

    fn handle_input(&mut self, input: OscillatorInput) {
        self.last_input = Some(input.clone());
        self.index_at_last_input = Some(self.index);

        match input {
            OscillatorInput::Press(freq) => {
                self.apply_change(OscillatorChange::Frequency(freq));
                self.secs_since_release = None;
            },
            OscillatorInput::Release => {
                self.index = 0;
                self.secs_since_release = Some(0.0);
            },
            OscillatorInput::PressSame => (),
        }
    }

    fn update_inputs(&mut self) {
        if let Some(input) = self.inputs.first() {
            if self.secs_since_start >= input.time {
                self.handle_input(input.input.clone());
                self.inputs.remove(0);
            }
        }
    }

    /// Apply the attack, decay, or sustain to a grain.
    fn apply_ads_to_grain(&mut self, grain: &mut [f32]) {
        let adsr = self.adsr.as_ref().unwrap();

        let decay_start = adsr.attack_duration;
        let sustain_start = decay_start + adsr.decay_duration;

        if self.secs_since_start < decay_start {
            // attack
            println!("attack ({})", self.secs_since_start);
            let attack_progress = self.secs_since_start / adsr.attack_duration;
            for sample in grain {
                *sample *= attack_progress;
            }
        } else if self.secs_since_start < sustain_start {
            // decay
            println!("decay ({})", self.secs_since_start);
            let decay_progress = (self.secs_since_start - decay_start) / adsr.decay_duration;
            let sustain_amplitude_multiplier = 1.0 - adsr.sustain_amplitude_multiplier * decay_progress;
            for sample in grain {
                *sample *= sustain_amplitude_multiplier;
            }
        } else {
            // sustain
            println!("sustain ({})", self.secs_since_start);
            for sample in grain {
                *sample *= adsr.sustain_amplitude_multiplier;
            }
        }
    }

    /// Get the next sample of the release phase of the ADSR.
    fn next_release_sample(&mut self) -> f32 {
        let adsr = self.adsr.as_ref().unwrap();

        // println!("secs_since_release: {}", self.secs_since_release.unwrap());
        let release_progress = self.secs_since_release.unwrap() / adsr.release_duration;
        // println!("release_progress: {}", release_progress);
        let release_amplitude_multiplier = adsr.sustain_amplitude_multiplier * (1.0 - release_progress);
        // println!("release_amplitude_multiplier: {}", release_amplitude_multiplier);

        let sample = self.wave_function.next_value(&mut self.phase, 1.0 / *SAMPLE_RATE as f32);
        sample * release_amplitude_multiplier
    }
}

impl Clone for Oscillator {
    fn clone(&self) -> Self {
        Self {
            wave_function: self.wave_function.clone(),
            index: self.index,
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
            phase: self.phase,
            last_input: self.last_input.clone(),
            index_at_last_input: self.index_at_last_input.clone(),
            inputs: self.inputs.clone(),
            secs_since_start: self.secs_since_start,
            secs_since_release: self.secs_since_release,
            adsr: self.adsr.clone(),
        }
    }
}

impl Sound for Oscillator {
    fn secs_per_beat(&self) -> Option<f32> {
        None
    }

    fn next_sample(&mut self) -> f32 {
        self.update_inputs();
        self.secs_since_start += 1.0 / *SAMPLE_RATE as f32;
        self.secs_since_release = self.secs_since_release.map(|secs| secs + 1.0 / *SAMPLE_RATE as f32);

        let dt = 1.0 / *SAMPLE_RATE as f32;

        if let Some(OscillatorInput::Release) = self.last_input {
            if let Some(adsr) = &self.adsr {
                if let Some(secs_since_release) = self.secs_since_release {
                    if secs_since_release <= adsr.release_duration {
                        let next_sample = self.next_release_sample();
                        // println!("release ({})", next_sample);

                        return next_sample;
                    }
                }
            }
            return 0.0;
        } else if self.last_input.is_none() {
            return 0.0;
        }

        self.index += 1;

        self.wave_function.next_value(&mut self.phase, dt)
    }

    fn next_grain(&mut self) -> Grain {
        let mut grain = [0.0; SAMPLES_PER_GRAIN];
        for sample in &mut grain {
            *sample = self.next_sample();
        }

        let mut oscillator_changes = Vec::new();
        for effect in &mut self.effects {
            let time_since_start_of_beat = self.index as f32 / *SAMPLE_RATE as f32;
            let input = EffectInput {
                grain,
                time_since_start_of_beat,
                time_since_release: self.secs_since_release,
            };
            let output = effect.apply(input);
            grain = output.grain;

            for change in output.oscillator_changes {
                oscillator_changes.push(change);
            }
        }

        if self.adsr.is_some() && self.secs_since_release.is_none() {
            self.apply_ads_to_grain(&mut grain)
        }

        for change in oscillator_changes {
            self.apply_change(change);
        }

        grain
    }

    fn update_sample_rate(&mut self, _sample_rate: usize) {} // does not affect anything

    fn clone_box(&self) -> Box<dyn Sound> {
        Box::new(Self {
            wave_function: self.wave_function.clone(),
            index: self.index,
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
            phase: self.phase,
            last_input: self.last_input.clone(),
            index_at_last_input: self.index_at_last_input.clone(),
            inputs: self.inputs.clone(),
            secs_since_start: self.secs_since_start,
            secs_since_release: self.secs_since_release,
            adsr: self.adsr.clone(),
        })
    }

    fn add_effect(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }
}

pub struct OscillatorBuilder {
    pub wave_function: Option<WaveFunction>,
    pub effects: Vec<Box<dyn Effect>>,
    pub inputs: Vec<OscillatorInputAtTime>,
    pub adsr: Option<ADSR>,
}

impl OscillatorBuilder {
    pub fn new() -> Self {
        Self {
            wave_function: None,
            effects: Vec::new(),
            inputs: Vec::new(),
            adsr: None,
        }
    }

    pub fn wave_function(mut self, wave_function: WaveFunction) -> Self {
        self.wave_function = Some(wave_function);
        self
    }

    pub fn effect(mut self, effect: Box<dyn Effect>) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn inputs(mut self, inputs: Vec<OscillatorInputAtTime>) -> Self {
        self.inputs.extend(inputs);
        self
    }

    pub fn auto_play(mut self) -> Self {
        self.inputs.push(OscillatorInputAtTime {
            input: OscillatorInput::PressSame,
            time: 0.0,
        });
        self
    }

    pub fn adsr(mut self, adsr: ADSR) -> Self {
        self.adsr = Some(adsr);
        self
    }

    pub fn build(self) -> Oscillator {
        // sort inputs by time
        let mut inputs = self.inputs;
        inputs.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        Oscillator {
            wave_function: Box::new(self.wave_function.unwrap()),
            index: 0,
            effects: self.effects,
            phase: 0.0,
            last_input: None,
            index_at_last_input: None,
            inputs,
            secs_since_start: 0.0,
            secs_since_release: None,
            adsr: self.adsr,
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
    WhiteNoise {
        amplitude: Number,
    },
    PinkNoise {
        amplitude: Number,
        generators: Vec<f32>,
        call_count: usize,
    },
}

impl WaveFunction {
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
            .auto_play()
            .build();

        Number::oscillator(oscillator).plus_f32(middle)
    }

    /// Create a square wave that oscillates around a middle value with a given frequency.
    pub fn square_around(middle: f32, plus_or_minus: f32, frequency: f32) -> Self {
        let oscillator = OscillatorBuilder::new()
            .wave_function(WaveFunction::Square {
                frequency: Number::number(frequency),
                amplitude: Number::number(plus_or_minus),
                phase: Number::number(0.0),
            })
            .auto_play()
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
