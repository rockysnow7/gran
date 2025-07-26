#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{LowPassFilter, Saturation, TapeDelay, Volume}, oscillator::{note, OscillatorBuilder, OscillatorInput, OscillatorInputAtTime, OscillatorInputIteratorBuilder, WaveFunction, ADSR}, play_sound, sample::{SampleBuilder, SampleInput, SampleInputAtTime, SampleInputIterator, SampleInputIteratorBuilder}, sound::CompositionBuilder, Number
};

#[tokio::main]
async fn main() {
    let bass = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(note("C4")),
            amplitude: Number::number(0.7),
            phase: Number::number(0.0),
        })
        .adsr(ADSR::new(0.4, 0.2, 0.5, 1.0))
        .effect(Box::new(Volume(Number::number(0.7))))
        .effect(Box::new(LowPassFilter::new(
            Number::sine_around(600.0, 200.0, 1.0),
            // Number::number(0.75),
            Number::square_around(0.3, 0.3, 2.0),
            4,
        )))
        .effect(Box::new(Saturation::new(
            Number::number(3.0),
            Number::number(0.5),
            0.5,
        )))
        .effect(Box::new(TapeDelay::light(0.5)))
        .inputs(OscillatorInputIteratorBuilder::new()
            .input(OscillatorInputAtTime {
                input: OscillatorInput::Press(note("C2")),
                time: 0.0,
            })
            .input(OscillatorInputAtTime {
                input: OscillatorInput::Press(note("E2")),
                time: 2.0,
            })
            .input(OscillatorInputAtTime {
                input: OscillatorInput::Press(note("G2")),
                time: 4.0,
            })
            .repeat_after(4.0)
            .build()
        )
        .build();

    let kick = SampleBuilder::new()
        .samples_from_file("samples/kick.wav")
        .secs_per_beat(1.0)
        .effect(Box::new(Volume(Number::number(100.0))))
        .inputs(SampleInputIteratorBuilder::new()
            .input(SampleInputAtTime {
                input: SampleInput::Trigger,
                time: 0.0,
            })
            .repeat_after(1.0)
            .build()
        )
        .build();

    let hat = SampleBuilder::new()
        .samples_from_file("samples/hat.wav")
        .secs_per_beat(0.5)
        .effect(Box::new(Volume(Number::number(100.0))))
        .inputs(SampleInputIteratorBuilder::new()
            .input(SampleInputAtTime {
                input: SampleInput::Trigger,
                time: 0.5,
            })
            .repeat_after(0.5)
            .build()
        )
        .build();

    let drums = CompositionBuilder::new()
        .sound(Box::new(kick))
        .sound(Box::new(hat))
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
        .sound(Box::new(drums))
        .build();

    play_sound(&mut composition);
}
