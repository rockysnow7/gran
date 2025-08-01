mod lfo;
mod input;

use crate::{effects::{Effect, EffectTrait, OscillatorChange}, player::SAMPLE_RATE, sound::{EffectInput, Grain, Sound, SAMPLES_PER_GRAIN}};
pub use lfo::{Number, WaveFunction};
pub use input::{OscillatorInput, OscillatorInputAtTime, OscillatorInputIterator, OscillatorInputIteratorBuilder};

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

/// Attack-decay-sustain-release envelope settings for an oscillator.
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum OscillatorState {
    Idle,
    Play {
        started_at: f32,
    },
    Release {
        started_at: f32,
    },
}

#[derive(Debug)]
pub struct Oscillator {
    wave_function: Box<WaveFunction>,
    index: usize,
    // effects: Vec<Box<dyn EffectTrait>>,
    effects: Vec<Effect>,
    phase: f32,
    // inputs: Vec<OscillatorInputAtTime>,
    inputs: OscillatorInputIterator,
    pub state: OscillatorState,
    secs_since_start: f32,
    adsr: ADSR,
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
        match input {
            OscillatorInput::Press(freq) => {
                self.apply_change(OscillatorChange::Frequency(freq));
                self.state = OscillatorState::Play { started_at: self.secs_since_start };
            },
            OscillatorInput::Release => {
                self.index = 0;
                self.state = OscillatorState::Release { started_at: self.secs_since_start };
            },
            OscillatorInput::PressSame => self.state = OscillatorState::Play { started_at: self.secs_since_start },
        }
    }

    fn update_inputs(&mut self) {
        if let Some(input) = self.inputs.next(self.secs_since_start) {
            self.handle_input(input.input);
        }
    }

    pub fn set_adsr(&mut self, adsr: ADSR) {
        self.adsr = adsr;
    }
}

impl Clone for Oscillator {
    fn clone(&self) -> Self {
        Self {
            wave_function: self.wave_function.clone(),
            index: self.index,
            // effects: self.effects.iter().map(|e| e.clone_box()).collect(),
            effects: self.effects.clone(),
            phase: self.phase,
            inputs: self.inputs.clone(),
            state: self.state.clone(),
            secs_since_start: self.secs_since_start,
            adsr: self.adsr.clone(),
        }
    }
}

impl Sound for Oscillator {
    fn secs_per_beat(&self) -> Option<f32> {
        None
    }

    fn next_sample(&mut self) -> f32 {
        self.secs_since_start += 1.0 / *SAMPLE_RATE as f32;

        // println!("state: {:?}", self.state);
        if let OscillatorState::Idle = &self.state {
            return 0.0;
        }

        self.index += 1;
        let dt = 1.0 / *SAMPLE_RATE as f32;

        self.wave_function.next_value(&mut self.phase, dt)
    }

    fn next_grain(&mut self) -> Grain {
        self.update_inputs();

        // get grain
        let mut grain = [0.0; SAMPLES_PER_GRAIN];
        for sample in &mut grain {
            *sample = self.next_sample();
        }

        // apply effects
        let mut oscillator_changes = Vec::new();
        for effect in &mut self.effects {
            let input = EffectInput {
                grain,
                time_since_start_of_beat: self.secs_since_start,
            };
            let output = effect.apply(input);
            grain = output.grain;

            for change in output.oscillator_changes {
                oscillator_changes.push(change);
            }
        }

        for change in oscillator_changes {
            self.apply_change(change);
        }

        // apply adsr
        match &self.state {
            OscillatorState::Idle => {},
            OscillatorState::Play { started_at } => {
                // attack/decay/sustain
                let secs_since_start_of_play = self.secs_since_start - started_at;

                let decay_start = self.adsr.attack_duration;
                let sustain_start = decay_start + self.adsr.decay_duration;

                if secs_since_start_of_play < decay_start {
                    // attack
                    let attack_progress = secs_since_start_of_play / self.adsr.attack_duration;
                    for sample in &mut grain {
                        *sample *= attack_progress;
                    }
                } else if secs_since_start_of_play < sustain_start {
                    // decay
                    let decay_progress = (secs_since_start_of_play - decay_start) / self.adsr.decay_duration;
                    let diff = 1.0 - self.adsr.sustain_amplitude_multiplier;
                    let amplitude = 1.0 - diff * decay_progress;
                    for sample in &mut grain {
                        *sample *= amplitude;
                    }
                } else {
                    // sustain
                    for sample in &mut grain {
                        *sample *= self.adsr.sustain_amplitude_multiplier;
                    }
                }
            },
            OscillatorState::Release { started_at } => {
                // release
                let secs_since_start_of_release = self.secs_since_start - started_at;
                if secs_since_start_of_release > self.adsr.release_duration {
                    self.state = OscillatorState::Idle;
                    grain = [0.0; SAMPLES_PER_GRAIN];
                } else {
                    let release_progress = secs_since_start_of_release / self.adsr.release_duration;
                    let amplitude = self.adsr.sustain_amplitude_multiplier * (1.0 - release_progress);
                    for sample in &mut grain {
                        *sample *= amplitude;
                    }
                }
            },
        }

        grain
    }

    fn update_sample_rate(&mut self, _sample_rate: usize) {} // does not affect anything

    fn clone_box(&self) -> Box<dyn Sound> {
        Box::new(Self {
            wave_function: self.wave_function.clone(),
            index: self.index,
            // effects: self.effects.iter().map(|e| e.clone_box()).collect(),
            effects: self.effects.clone(),
            phase: self.phase,
            inputs: self.inputs.clone(),
            state: self.state.clone(),
            secs_since_start: self.secs_since_start,
            adsr: self.adsr.clone(),
        })
    }

    fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}

pub struct OscillatorBuilder {
    pub wave_function: Option<WaveFunction>,
    // pub effects: Vec<Box<dyn EffectTrait>>,
    pub effects: Vec<Effect>,
    pub inputs: Option<OscillatorInputIterator>,
    pub adsr: Option<ADSR>,
}

impl OscillatorBuilder {
    pub fn new() -> Self {
        Self {
            wave_function: None,
            effects: Vec::new(),
            inputs: None,
            adsr: None,
        }
    }

    pub fn wave_function(mut self, wave_function: WaveFunction) -> Self {
        self.wave_function = Some(wave_function);
        self
    }

    pub fn effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn inputs(mut self, inputs: OscillatorInputIterator) -> Self {
        self.inputs = Some(inputs);
        self
    }

    pub fn adsr(mut self, adsr: ADSR) -> Self {
        self.adsr = Some(adsr);
        self
    }

    pub fn build(self) -> Oscillator {
        let adsr = self.adsr.unwrap_or(ADSR::new(0.1, 0.1, 1.0, 0.1));

        Oscillator {
            wave_function: Box::new(self.wave_function.unwrap()),
            index: 0,
            effects: self.effects,
            phase: 0.0,
            inputs: self.inputs.unwrap(),
            state: OscillatorState::Idle,
            secs_since_start: 0.0,
            adsr,
        }
    }
}
