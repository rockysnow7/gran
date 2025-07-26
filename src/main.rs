#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{LowPassFilter, Saturation, TapeDelay, Volume},
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
        .adsr(ADSR::new(1.0, 0.2, 0.5, 1.0))
        .effect(Box::new(Volume(Number::number(0.7))))
        .effect(Box::new(LowPassFilter::new(
            Number::sine_around(600.0, 200.0, 1.0),
            Number::number(0.75),
            4,
        )))
        // .effect(Box::new(Saturation::new(
        //     Number::number(3.0),
        //     Number::number(0.5),
        //     0.5,
        // )))
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C2")),
                time: 0.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("E2")),
                time: 2.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("G2")),
                time: 4.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C2")),
                time: 8.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("E2")),
                time: 10.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("G2")),
                time: 12.0,
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
        .effect(Box::new(TapeDelay::light(0.5)))
        .build();

    play_sound(&mut composition);
}
