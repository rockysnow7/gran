#[derive(Debug, Clone, Copy)]
pub enum SampleInput {
    Trigger,
}

#[derive(Debug, Clone, Copy)]
pub struct SampleInputAtTime {
    pub input: SampleInput,
    pub time: f32,
}

#[derive(Clone)]
pub struct SampleInputIterator {
    inputs: Vec<SampleInputAtTime>,
    index: usize,
    total_duration: f32,
    repeat_delay: Option<f32>, // in seconds
}

impl SampleInputIterator {
    pub fn new(inputs: Vec<SampleInputAtTime>, repeat_delay: Option<f32>) -> Self {
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
            for SampleInputAtTime { time, .. } in self.inputs.iter_mut() {
                *time += self.total_duration + delay;
            }

            self.index = 0;
        }
    }

    pub fn next(&mut self, secs_since_start: f32) -> Option<SampleInputAtTime> {
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

pub struct SampleInputIteratorBuilder {
    inputs: Vec<SampleInputAtTime>,
    repeat_delay: Option<f32>,
}

impl SampleInputIteratorBuilder {
    pub fn new() -> Self {
        Self { inputs: vec![], repeat_delay: None }
    }

    pub fn input(mut self, input: SampleInputAtTime) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn repeat_after(mut self, delay: f32) -> Self {
        self.repeat_delay = Some(delay);
        self
    }

    pub fn build(self) -> SampleInputIterator {
        SampleInputIterator::new(self.inputs, self.repeat_delay)
    }
}
