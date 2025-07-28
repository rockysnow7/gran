#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Saturation, TapeDelay, Volume}, oscillator::{note, OscillatorBuilder, OscillatorInput, OscillatorInputAtTime, OscillatorInputIteratorBuilder, WaveFunction, ADSR}, play_sound, sample::{SampleBuilder, SampleInput, SampleInputAtTime, SampleInputIterator, SampleInputIteratorBuilder}, sound::CompositionBuilder, Number
};

fn main() {
    let inputs = OscillatorInputIteratorBuilder::new()
        .input(OscillatorInputAtTime {
            input: OscillatorInput::Press(note("C3")),
            time: 0.0,
        })
        .input(OscillatorInputAtTime {
            input: OscillatorInput::Release,
            time: 0.3,
        })
        .input(OscillatorInputAtTime {
            input: OscillatorInput::Press(note("E3")),
            time: 0.5,
        })
        .input(OscillatorInputAtTime {
            input: OscillatorInput::Release,
            time: 0.75,
        })
        .input(OscillatorInputAtTime {
            input: OscillatorInput::Press(note("E3")),
            time: 1.0,
        })
        .input(OscillatorInputAtTime {
            input: OscillatorInput::Release,
            time: 1.25,
        })
        .input(OscillatorInputAtTime {
            input: OscillatorInput::Press(note("E3")),
            time: 1.5,
        })
        .input(OscillatorInputAtTime {
            input: OscillatorInput::Release,
            time: 1.75,
        })
        .repeat_after(0.25)
        .build();

    let bass = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(0.0),
            amplitude: Number::number(1.0),
            phase: Number::number(0.0),
        })
        .adsr(ADSR {
            attack_duration: 0.2,
            decay_duration: 0.05,
            sustain_amplitude_multiplier: 0.8,
            release_duration: 0.3,
        })
        .effect(Box::new(Volume(Number::number(1.0))))
        .effect(Box::new(Filter::new_low_pass(
            Number::sine_around(600.0, 50.0, 2.0),
            Number::number(0.5),
            4,
        )))
        .effect(Box::new(Saturation::new(Number::number(8.0), Number::number(1.0), 1.0)))
        .inputs(inputs)
        .build();

    let pink_noise = OscillatorBuilder::new()
        .wave_function(WaveFunction::pink_noise(Number::number(0.005), 10))
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
        .effect(Box::new(TapeDelay::light(0.05)))
        .build();

    play_sound(&mut composition);
}
