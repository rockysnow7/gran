#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Pattern, PatternBeat, Volume}, oscillators::{Number, OscillatorBuilder, WaveFunction}, play_sound, sounds::{CompositionBuilder, SampleBuilder}
};

#[tokio::main]
async fn main() {
    let kick = SampleBuilder::new()
        .samples_from_file("samples/kick.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Volume(Number::number(100.0))))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::number(2.5)),
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::number(2.5)),
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::number(2.5)),
            PatternBeat::Skip,
        ]).humanize(0.5)))
        .build();

    let hat = SampleBuilder::new()
        .samples_from_file("samples/hat.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Volume(Number::number(75.0))))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::number(0.8)),
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::number(2.0)),
            PatternBeat::Play,
            PatternBeat::PlayWithVolume(Number::number(0.8)),
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::PlayWithVolume(Number::number(2.0)),
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Play,
        ]).humanize(0.2)))
        .build();

    let drums = CompositionBuilder::new()
        .sound(Box::new(kick))
        .sound(Box::new(hat))
        .build();

    let sine_frequency = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sine {
            frequency: Number::number(1.0), // 5Hz vibrato
            amplitude: Number::number(5.0), // +/- 5Hz
            phase: Number::number(0.0),
        })
        .beat_length(1.0)
        .build();
    let sine_frequency = Number::oscillator(sine_frequency).plus_f32(220.0);

    let sawtooth = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: sine_frequency,
            amplitude: Number::number(1.0),
            phase: Number::number(0.0),
        })
        .beat_length(1.0)
        .effect(Box::new(Volume(Number::number(0.1))))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Skip,
            PatternBeat::Skip,
        ])))
        .build();

    let mut full = CompositionBuilder::new()
        .sound(Box::new(drums))
        .sound(Box::new(sawtooth))
        .build();

    play_sound(&mut full);
}
