use std::{collections::{HashMap, VecDeque}, sync::atomic::{AtomicUsize, Ordering}};

const DEFAULT_SAMPLE_RATE: usize = 48000;
const AMPLIFICATION_FACTOR: f32 = 100.0;
const GRAIN_SIZE_SECONDS: f32 = 0.003; // 3ms per grain

#[derive(Debug, Clone)]
pub struct PatternConfig {
    pub bpm: u16,
    pub volume: f32,
    pub length_beats: u8, // the number of beats in the pattern
}

impl PatternConfig {
    pub fn new(bpm: u16, volume: f32, length_beats: u8) -> Self {
        Self { bpm, volume, length_beats }
    }
}

/// A history of dry samples for effects to use.
#[derive(Debug, Clone)]
pub struct History {
    samples: VecDeque<f32>,
    samples_per_grain: usize,
}

impl History {
    pub fn new(samples_per_grain: usize) -> Self {
        let size = samples_per_grain * 2;
        let mut samples = VecDeque::with_capacity(size);
        samples.extend(vec![0.0; size]);

        Self {
            samples,
            samples_per_grain,
        }
    }

    pub fn push(&mut self, sample: f32) {
        self.samples.pop_front();
        self.samples.push_back(sample);
    }

    pub fn last_grain(&self) -> Vec<&f32> {
        self.samples.iter().take(self.samples_per_grain).collect()
    }
}

/// An `Effect` is a function that is applied to the granular history of a pattern and returns a new sample.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Apply a function to the history.
    Fn(fn(&History) -> f32),
    /// Amplify the sample by a factor.
    Amplify(f32),
    /// Make the sample more crunchy.
    Crunchy(f32),
    /// Shift the pitch of the sample by a given number of semitones.
    PitchShift(i8),
}

impl Effect {
    pub fn apply(&self, history: &History) -> f32 {
        let result = match self {
            Effect::Fn(f) => f(history),
            Effect::Amplify(factor) => {
                let last_grain = history.last_grain();
                let most_recent_sample = last_grain.last().unwrap_or(&&0.0);
                // println!("most_recent_sample: {}", most_recent_sample);
                let amplified = *most_recent_sample * factor;

                amplified
            },
            Effect::Crunchy(decay) => {
                let last_grain = history.last_grain();
                
                // Get the current (most recent) sample
                let current_sample = last_grain.first().unwrap_or(&&0.0);
                
                // Calculate crunchy contribution from older samples
                // Use decay clamped between 0.1 and 0.95 for stability
                let safe_decay = decay.clamp(0.1, 0.95);
                let crunchy_contribution = last_grain.iter()
                    .skip(1) // Skip the current sample
                    .enumerate()
                    .map(|(i, &sample)| sample * safe_decay.powi((i + 1) as i32))
                    .sum::<f32>();

                // Mix the current sample with crunchy (decay also controls mix level)
                **current_sample + crunchy_contribution * safe_decay * 0.5
            },
            Effect::PitchShift(semitones) => {
                todo!()
            }
        };

        result.clamp(-1.0, 1.0)
    }
}

#[derive(Debug)]
pub struct Pattern {
    history: History,
    samples_per_grain: usize,
    config: PatternConfig,
    sample: Vec<f32>,
    trigger_beats: Vec<u8>,
    effects: Vec<Effect>,
    sample_counter: AtomicUsize,
    current_sample_rate: usize,
    samples_per_beat: usize,
    samples_per_pattern: usize,
    trigger_map: Vec<bool>,
}

impl Clone for Pattern {
    fn clone(&self) -> Self {
        Pattern {
            history: self.history.clone(),
            samples_per_grain: self.samples_per_grain,
            config: self.config.clone(),
            sample: self.sample.clone(),
            trigger_beats: self.trigger_beats.clone(),
            effects: self.effects.clone(),
            sample_counter: AtomicUsize::new(self.sample_counter.load(Ordering::Relaxed)),
            current_sample_rate: self.current_sample_rate,
            samples_per_beat: self.samples_per_beat,
            samples_per_pattern: self.samples_per_pattern,
            trigger_map: self.trigger_map.clone(),
        }
    }
}

impl Pattern {
    pub fn new(
        config: PatternConfig,
        sample: Vec<f32>,
        trigger_beats: Vec<u8>,
        effects: Vec<Effect>,
    ) -> Self {
        Self::new_with_sample_rate(config, sample, trigger_beats, effects, DEFAULT_SAMPLE_RATE)
    }

    pub fn new_with_sample_rate(
        config: PatternConfig,
        sample: Vec<f32>,
        trigger_beats: Vec<u8>,
        effects: Vec<Effect>,
        sample_rate: usize,
    ) -> Self {
        // pad sample to full beat length
        let seconds_per_beat = 60.0 / config.bpm as f32;
        let samples_per_beat = (seconds_per_beat * sample_rate as f32) as usize;
        let samples_per_pattern = samples_per_beat * config.length_beats as usize;
        
        let mut sample = sample.clone();
        if sample.len() < samples_per_beat {
            sample.extend(vec![0.0; samples_per_beat - sample.len()]);
        }

        let sample = sample.iter()
            .map(|&s| s * config.volume * AMPLIFICATION_FACTOR)
            .map(|s| s.clamp(-1.0, 1.0))
            .collect();

        // pre-calculate trigger map for O(1) lookup
        let mut trigger_map = vec![false; config.length_beats as usize];
        for &beat in &trigger_beats {
            if beat > 0 && beat <= config.length_beats {
                trigger_map[(beat - 1) as usize] = true;
            }
        }

        let samples_per_grain = (GRAIN_SIZE_SECONDS * sample_rate as f32) as usize;
        let history = History::new(samples_per_grain);

        Self {
            history,
            samples_per_grain,
            config,
            sample,
            trigger_beats,
            effects,
            sample_counter: AtomicUsize::new(0),
            current_sample_rate: sample_rate,
            samples_per_beat,
            samples_per_pattern,
            trigger_map,
        }
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: usize) {
        if new_sample_rate != self.current_sample_rate {
            self.current_sample_rate = new_sample_rate;
            
            // recalculate timing values
            let seconds_per_beat = 60.0 / self.config.bpm as f32;
            self.samples_per_beat = (seconds_per_beat * new_sample_rate as f32) as usize;
            self.samples_per_pattern = self.samples_per_beat * self.config.length_beats as usize;

            // re-calculate samples_per_grain
            self.samples_per_grain = (GRAIN_SIZE_SECONDS * new_sample_rate as f32) as usize;
            
            // re-pad sample if needed
            if self.sample.len() < self.samples_per_beat {
                self.sample.extend(vec![0.0; self.samples_per_beat - self.sample.len()]);
            }
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let position = self.sample_counter.fetch_add(1, Ordering::Relaxed);
        self.get_sample_at_position(position)
    }

    #[inline]
    pub fn get_sample_at_position(&mut self, sample_position: usize) -> f32 {
        let pattern_position = sample_position % self.samples_per_pattern;
        let beat_position = pattern_position / self.samples_per_beat;
        
        let sample = if beat_position < self.trigger_map.len() && self.trigger_map[beat_position] {
            let position_in_beat = pattern_position - (beat_position * self.samples_per_beat);
            
            if self.sample.len() > 0 {
                let sample_index = (position_in_beat * self.sample.len()) / self.samples_per_beat;
                if sample_index < self.sample.len() {
                    self.sample[sample_index]
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };
        self.history.push(sample);

        let mut processed_sample = sample;
        for effect in &self.effects {
            processed_sample = effect.apply(&self.history);
        }

        processed_sample
    }
}

pub struct PatternBuilder {
    config: Option<PatternConfig>,
    sample: Option<Vec<f32>>,
    trigger_beats: Vec<u8>,
    effects: Vec<Effect>,
}

impl PatternBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            sample: None,
            trigger_beats: vec![],
            effects: vec![],
        }
    }

    pub fn config(&mut self, config: PatternConfig) -> &mut Self {
        self.config = Some(config);
        self
    }

    pub fn sample(&mut self, sample: Vec<f32>) -> &mut Self {
        self.sample = Some(sample);
        self
    }

    pub fn trigger_beats(&mut self, trigger_beats: Vec<u8>) -> &mut Self {
        self.trigger_beats = trigger_beats;
        self
    }

    pub fn effect(&mut self, effect: Effect) -> &mut Self {
        self.effects.push(effect);
        self
    }

    pub fn build(&self) -> Result<Pattern, String> {
        if self.config.is_none() || self.sample.is_none() {
            return Err("Missing required fields".to_string());
        }

        Ok(Pattern::new(
            self.config.clone().unwrap(),
            self.sample.clone().unwrap(),
            self.trigger_beats.clone(),
            self.effects.clone(),
        ))
    }
}

#[derive(Debug)]
pub struct Composition {
    pub patterns: HashMap<String, Pattern>,
}

impl Composition {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    pub fn add_pattern(&mut self, name: String, pattern: Pattern) {
        self.patterns.insert(name, pattern);
    }
}
