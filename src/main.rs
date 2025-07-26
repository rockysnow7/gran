#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Saturation, TapeDelay, Volume},
    oscillators::{note, ADSR, OscillatorBuilder, OscillatorInput, OscillatorInputAtTime, WaveFunction},
    play_sound, Number,
    sounds::CompositionBuilder,
};

#[tokio::main]
async fn main() {
    let bass = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(note("C4")),
            amplitude: Number::number(0.7),
            phase: Number::number(0.0),
        })
        // .adsr(ADSR::new(0.1, 0.1, 0.5, 1.0))
        .effect(Box::new(Volume(Number::number(0.7))))
        .effect(Box::new(Filter::low_pass(
            // Number::number(300.0),
            Number::sine_around(300.0, 200.0, 1.0),
            Number::number(0.5),
        )))
        .effect(Box::new(Saturation::new(
            Number::number(4.0),
            Number::number(0.5),
            0.4,
        )))
        .effect(Box::new(TapeDelay::light(1.0)))
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C2")),
                time: 0.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 4.0,
            },
        ])
        .build();

    let pink_noise = OscillatorBuilder::new()
        .wave_function(WaveFunction::pink_noise(Number::number(0.001), 10))
        .auto_play()
        .build();

    let mut composition = CompositionBuilder::new()
        .sound(Box::new(pink_noise))
        .sound(Box::new(bass))
        .build();

    play_sound(&mut composition);
}
