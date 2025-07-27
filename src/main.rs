#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{LowPassFilter, Saturation, TapeDelay, Volume}, oscillator::{note, OscillatorBuilder, OscillatorInput, OscillatorInputAtTime, OscillatorInputIteratorBuilder, WaveFunction, ADSR}, play_sound, sample::{SampleBuilder, SampleInput, SampleInputAtTime, SampleInputIterator, SampleInputIteratorBuilder}, sound::CompositionBuilder, Number
};

#[tokio::main]
async fn main() {
    let bass = OscillatorBuilder::new()
        .wave_function(WaveFunction::Square {
            // frequency: Number::number(note("C2")),
            frequency: Number::sine_around(3000.0, 2800.0, 0.2),
            amplitude: Number::number(0.2),
            phase: Number::number(0.0),
        })
        .inputs(OscillatorInputIteratorBuilder::new()
            .input(OscillatorInputAtTime {
                input: OscillatorInput::PressSame,
                time: 0.0,
            })
            .build()
        )
        .build();

    let pink_noise = OscillatorBuilder::new()
        .wave_function(WaveFunction::pink_noise(Number::number(0.0005), 10))
        .inputs(OscillatorInputIteratorBuilder::new()
            .input(OscillatorInputAtTime {
                input: OscillatorInput::PressSame,
                time: 0.0,
            })
            .build()
        )
        .build();

    let mut composition = CompositionBuilder::new()
        .sound(Box::new(pink_noise))
        .sound(Box::new(bass))
        .build();

    play_sound(&mut composition);
}
