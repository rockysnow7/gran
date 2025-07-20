#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Pattern, PatternBeat, Volume}, play_composition, sound::{Composition, SampleBuilder}
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

    let composition = Composition(vec![Box::new(kick), Box::new(hat)]);

    play_composition(&composition);
}
