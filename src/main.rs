#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Saturation, TapeDelay, Volume}, oscillator::{note, OscillatorBuilder, OscillatorInput, OscillatorInputAtTime, OscillatorInputIteratorBuilder, WaveFunction, ADSR}, play_sound, sample::{SampleBuilder, SampleInput, SampleInputAtTime, SampleInputIterator, SampleInputIteratorBuilder}, sound::CompositionBuilder, Number
};

#[tokio::main]
async fn main() {
    let bass = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(0.0),
            amplitude: Number::number(1.0),
            phase: Number::number(0.0),
        })
        // .wave_function(WaveFunction::white_noise(Number::number(0.1)))
        .effect(Box::new(Volume(Number::number(1.5))))
        // .effect(Box::new(Filter::new_notch(
        //     Number::sine_around(10010.0, 19980.0, 0.2),
        //     // Number::number(2000.0),
        //     Number::number(1.0),
        //     4,
        // )))
        .effect(Box::new(Saturation::new(
            Number::number(1.0),
            Number::number(0.5),
            1.0,
        )))
        .inputs(OscillatorInputIteratorBuilder::new()
            .input(OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C3")),
                time: 0.0,
            })
            // .input(OscillatorInputAtTime {
            //     input: OscillatorInput::Press(note("E2")),
            //     time: 2.0,
            // })
            // .input(OscillatorInputAtTime {
            //     input: OscillatorInput::Press(note("G2")),
            //     time: 4.0,
            // })
            // .repeat_after(4.0)
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
        // .sound(Box::new(pink_noise))
        .sound(Box::new(bass))
        // .effect(Box::new(TapeDelay::light(0.5)))
        .build();

    play_sound(&mut composition);
}
