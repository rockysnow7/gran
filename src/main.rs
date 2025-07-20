#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    play_composition,
    sound::{Composition, SampleBuilder},
    effects::{Gain, Pattern},
};

#[tokio::main]
async fn main() {
    let kick = SampleBuilder::new()
        .samples_from_file("samples/kick.wav")
        .secs_per_beat(0.5)
        .effect(Box::new(Gain(100.0)))
        .effect(Box::new(Pattern {
            trigger_beats: vec![0, 1, 3],
            length: 4,
        }))
        .build();

    let hat = SampleBuilder::new()
        .samples_from_file("samples/hat.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Gain(100.0)))
        .build();

    let strings = SampleBuilder::new()
        .samples_from_file("samples/strings.mp3")
        .secs_per_beat(0.5)
        .effect(Box::new(Gain(1500.0)))
        .effect(Box::new(Pattern {
            trigger_beats: vec![0],
            length: 2,
        }))
        .build();

    let mut composition = Composition::new();
    composition.add_sound("kick".to_string(), Box::new(kick));
    composition.add_sound("hat".to_string(), Box::new(hat));
    composition.add_sound("strings".to_string(), Box::new(strings));

    play_composition(&composition);
}
