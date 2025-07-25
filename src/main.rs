#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Saturation, Volume}, 
    oscillators::{note, Number, OscillatorBuilder, OscillatorInput, OscillatorInputAtTime, WaveFunction}, 
    play_sound, sounds::{CompositionBuilder, SampleBuilder, SampleInput, SampleInputAtTime},
};

#[tokio::main]
async fn main() {
    let a = osc()
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("B3")),
                time: 0.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 1.0,
            },
        ])
        .build();

    let kick = SampleBuilder::new()
        .samples_from_file("samples/kick.wav")
        .secs_per_beat(0.5)
        .effect(Box::new(Volume(Number::number(300.0))))
        .inputs(vec![
            SampleInputAtTime {
                input: SampleInput::Trigger,
                time: 0.0,
            },
        ])
        .build();

    let mut composition = CompositionBuilder::new()
        .sound(Box::new(a))
        .sound(Box::new(kick))
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
            Number::number(300.0),
            Number::number(0.5),
        )))
        .effect(Box::new(Saturation::new(
            Number::number(7.0),
            Number::number(0.5),
            0.4,
        )))
}
