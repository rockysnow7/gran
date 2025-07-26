/// An input to an oscillator. Like a simplified form of MIDI.
#[derive(Debug, Clone, Copy)]
pub enum OscillatorInput {
    Press(f32), // frequency in Hz
    PressSame, // press the same frequency as the last input
    Release,
}

/// An input to be sent to an oscillator at a given time.
#[derive(Debug, Clone, Copy)]
pub struct OscillatorInputAtTime {
    pub input: OscillatorInput,
    pub time: f32, // in seconds since the start of the oscillator
}

#[derive(Clone)]
pub struct OscillatorInputIterator {
    inputs: Vec<OscillatorInputAtTime>,
    index: usize,
    total_duration: f32,
    repeat_delay: Option<f32>, // in seconds
}

impl OscillatorInputIterator {
    pub fn new(inputs: Vec<OscillatorInputAtTime>, repeat_delay: Option<f32>) -> Self {
        let total_duration = inputs.last().unwrap().time;

        Self {
            inputs,
            index: 0,
            total_duration,
            repeat_delay,
        }
    }

    fn repeat_inputs(&mut self) {
        if let Some(delay) = self.repeat_delay {
            for OscillatorInputAtTime { time, .. } in self.inputs.iter_mut() {
                *time += self.total_duration + delay;
            }

            self.index = 0;
        }
    }

    pub fn next(&mut self, secs_since_start: f32) -> Option<OscillatorInputAtTime> {
        if self.index >= self.inputs.len() {
            return None;
        }

        let index_input = self.inputs[self.index];
        let next_input = if secs_since_start >= index_input.time {
            self.index += 1;
            if self.index >= self.inputs.len() {
                self.repeat_inputs();
            }

            Some(index_input)
        } else {
            None
        };

        next_input
    }
}

pub struct OscillatorInputIteratorBuilder {
    inputs: Vec<OscillatorInputAtTime>,
    repeat_delay: Option<f32>,
}

impl OscillatorInputIteratorBuilder {
    pub fn new() -> Self {
        Self { inputs: vec![], repeat_delay: None }
    }

    pub fn input(mut self, input: OscillatorInputAtTime) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn repeat_after(mut self, delay: f32) -> Self {
        self.repeat_delay = Some(delay);
        self
    }

    pub fn build(self) -> OscillatorInputIterator {
        OscillatorInputIterator::new(self.inputs, self.repeat_delay)
    }
}
