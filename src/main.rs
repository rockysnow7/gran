#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Saturation, TapeDelay, Volume}, 
    oscillators::{note, Number, OscillatorBuilder, OscillatorInput, OscillatorInputAtTime, WaveFunction}, 
    play_sound, sounds::{CompositionBuilder, SampleBuilder, SampleInput, SampleInputAtTime},
};

#[tokio::main]
async fn main() {
    let a = osc()
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C3")),
                time: 0.0,
            },
        ])
        .build();

    let b = osc()
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("E3")),
                time: 0.02,
            },
        ])
        .build();

    let c = osc()
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("G3")),
                // time: 0.04,
                time: 0.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 0.5,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("G#3")),
                time: 0.6,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 2.6,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("G3")),
                time: 2.7,
            },
        ])
        .build();

    let mut composition = CompositionBuilder::new()
        .sound(Box::new(a))
        .sound(Box::new(b))
        .sound(Box::new(c))
        .effect(Box::new(Volume(Number::number(0.8))))
        .build();

    play_sound(&mut composition);
}

fn osc() -> OscillatorBuilder {
    OscillatorBuilder::new()
        .wave_function(WaveFunction::Sine {
            frequency: Number::number(note("B3")),
            amplitude: Number::number(0.7),
            phase: Number::number(0.0),
        })
        .effect(Box::new(Volume(Number::number(0.7))))
        .effect(Box::new(Filter::low_pass(
            Number::number(400.0),
            Number::number(0.5),
        )))
        .effect(Box::new(Saturation::new(
            Number::number(7.0),
            Number::number(0.5),
            0.4,
        )))
        .effect(Box::new(TapeDelay::light(0.5)))
}
