#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Pattern, PatternBeat, Volume, ADSR}, oscillators::{Number, OscillatorBuilder, WaveFunction}, play_sound, sounds::{CompositionBuilder, SampleBuilder}
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
        ]).humanize(0.2)))
        .build();

    let hat = SampleBuilder::new()
        .samples_from_file("samples/hat.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Volume(Number::number(75.0))))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::number(0.8)),
            PatternBeat::PlayWithVolume(Number::number(0.2)),
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

    let lead_filter = Filter::low_pass(
        Number::sine_around(400.0, 200.0, 0.5),
        Number::number(0.9),
    );

    let lead = OscillatorBuilder::new()
        .wave_function(WaveFunction::Triangle {
            frequency: Number::sine_around(220.0, 5.0, 5.0),
            amplitude: Number::number(1.0),
            phase: Number::number(0.0),
        })
        .beat_length(1.0)
        .effect(Box::new(Volume(Number::number(0.2))))
        .effect(Box::new(ADSR::new(0.1, 0.1, 0.7, 0.2)))
        .effect(Box::new(lead_filter))
        .build();

    let mut full = CompositionBuilder::new()
        .sound(Box::new(drums))
        .sound(Box::new(lead))
        .effect(Box::new(Volume(Number::number(0.5))))
        .build();

    play_sound(&mut full);
}
