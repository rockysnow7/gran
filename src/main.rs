#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Pattern, PatternBeat, Volume}, oscillators::{waves, Number, OscillatorBuilder}, play_sound, sounds::{CompositionBuilder, SampleBuilder, Sound}
};

#[tokio::main]
async fn main() {
    let kick = SampleBuilder::new()
        .samples_from_file("samples/kick.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Volume(Number::Number(100.0))))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::Number(2.5)),
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::Number(2.5)),
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::Number(2.5)),
            PatternBeat::Skip,
        ]).humanize(0.5)))
        .build();

    let hat = SampleBuilder::new()
        .samples_from_file("samples/hat.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Volume(Number::Number(75.0))))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::Number(0.8)),
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::Number(2.0)),
            PatternBeat::Play,
            PatternBeat::PlayWithVolume(Number::Number(0.8)),
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::PlayWithVolume(Number::Number(2.0)),
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Play,
        ]).humanize(0.2)))
        .build();

    let drums = CompositionBuilder::new()
        .sound(Box::new(kick))
        .sound(Box::new(hat))
        .build();

    let sine_volume = OscillatorBuilder::new()
        .function(waves::sine(1.0))
        .beat_length(1.0)
        .build();
    let sine_volume = Number::Oscillator(sine_volume)
        .mul(10.0)
        .plus(5.0);

    let sine = OscillatorBuilder::new()
        .function(waves::sine(220.0))
        .beat_length(1.0)
        .effect(Box::new(Volume(sine_volume.mul(0.03))))
        .build();

    let mut full = CompositionBuilder::new()
        .sound(Box::new(drums))
        .sound(Box::new(sine))
        .build();

    play_sound(&mut full);
}
