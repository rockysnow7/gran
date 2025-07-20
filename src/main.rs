#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Pattern, PatternBeat, Volume}, play_sound, sound::{CompositionBuilder, SampleBuilder}
};

#[tokio::main]
async fn main() {
    let kick = SampleBuilder::new()
        .samples_from_file("samples/kick.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Volume(100.0)))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(2.5),
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(2.5),
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(2.5),
            PatternBeat::Skip,
        ]).humanize(0.5)))
        .build();

    let hat = SampleBuilder::new()
        .samples_from_file("samples/hat.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Volume(75.0)))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(0.8),
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(2.0),
            PatternBeat::Play,
            PatternBeat::PlayWithVolume(0.8),
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::PlayWithVolume(2.0),
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Play,
        ]).humanize(0.2)))
        .build();

    let drums = CompositionBuilder::new()
        .sound(Box::new(kick))
        .sound(Box::new(hat))
        .build();

    let strings = SampleBuilder::new()
        .samples_from_file("samples/strings.mp3")
        .secs_per_beat(1.0)
        .effect(Box::new(Volume(400.0)))
        .build();

    let mut full = CompositionBuilder::new()
        .sound(Box::new(drums))
        .sound(Box::new(strings))
        .effect(Box::new(Volume(0.8)))
        .build();

    play_sound(&mut full);
}
