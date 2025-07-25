#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Saturation, TapeDelay, Volume}, 
    oscillators::{note, ADSR, Number, OscillatorBuilder, OscillatorInput, OscillatorInputAtTime, WaveFunction}, 
    play_sound, sounds::{CompositionBuilder, SampleBuilder, SampleInput, SampleInputAtTime},
};

#[tokio::main]
async fn main() {
    let a = osc_1()
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("F4")),
                time: 0.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("F5")),
                time: 1.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 3.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C#5")),
                time: 3.1,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 5.3,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("F#4")),
                time: 5.6,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 8.8,
            },
        ])
        .build();

    let b = osc_1()
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("G#4")),
                time: 0.02,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("G#5")),
                time: 1.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 3.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("F5")),
                time: 3.2,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 5.3,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("A4")),
                time: 5.6,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 8.8,
            },
        ])
        .build();

    let c = osc_2()
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C4")),
                time: 6.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 6.2,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C#4")),
                time: 6.4,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 6.6,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C4")),
                time: 7.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 7.3,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("F4")),
                time: 10.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("A#5")),
                time: 10.4,
            }
        ])
        .build();

    let d = osc_2()
        .inputs(vec![
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("D#4")),
                time: 6.1,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 6.3,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("F4")),
                time: 6.4,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 6.5,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("D#4")),
                time: 7.0,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Release,
                time: 7.3,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("G#4")),
                time: 10.1,
            },
            OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C#5")),
                time: 10.4,
            },
        ])
        .build();

    let pink_noise = OscillatorBuilder::new()
        .wave_function(WaveFunction::pink_noise(Number::number(0.0015), 10))
        .auto_play()
        .build();

    let mut composition = CompositionBuilder::new()
        .sound(Box::new(pink_noise))
        .sound(Box::new(a))
        // .sound(Box::new(b))
        // .sound(Box::new(c))
        // .sound(Box::new(d))
        .effect(Box::new(Volume(Number::number(0.8))))
        .build();

    play_sound(&mut composition);
}

fn osc_1() -> OscillatorBuilder {
    OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(note("C4")),
            amplitude: Number::number(0.7),
            phase: Number::number(0.0),
        })
        .effect(Box::new(Volume(Number::number(0.7))))
        .effect(Box::new(Filter::low_pass(
            Number::number(300.0),
            Number::number(0.5),
        )))
        .effect(Box::new(Saturation::new(
            Number::number(3.0),
            Number::number(0.5),
            0.4,
        )))
        .adsr(ADSR::new(0.3, 0.2, 0.5, 1.0))
        .effect(Box::new(TapeDelay::light(0.5)))
}

fn osc_2() -> OscillatorBuilder {
    OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(note("C4")),
            amplitude: Number::number(0.7),
            phase: Number::number(0.0),
        })
        .adsr(ADSR::new(0.3, 0.2, 0.5, 1.0))
        .effect(Box::new(Volume(Number::number(0.3))))
        .effect(Box::new(Filter::low_pass(
            Number::sine_around(800.0, 100.0, 0.1),
            Number::number(0.8),
        )))
        .effect(Box::new(Saturation::new(
            Number::number(10.0),
            Number::number(0.5),
            0.4,
        )))
}
