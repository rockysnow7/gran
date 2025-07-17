mod player;
mod state;

use player::play_composition;
use rodio::Decoder;
use state::{Composition, Effect, PatternBuilder, PatternConfig};
use std::{fs::File, io::BufReader};

fn load_sample_wav(path: &str) -> Vec<f32> {
    let mut reader = hound::WavReader::open(path).unwrap();
    let samples: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();

    samples.iter().map(|s| *s as f32 / i32::MAX as f32).collect()
}

fn load_sample_mp3(path: &str) -> Vec<f32> {
    let file = File::open(path).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();
    
    let samples: Vec<f32> = source
        .into_iter()
        .map(|sample| sample as f32 / i16::MAX as f32)
        .collect();
    
    samples
}

#[tokio::main]
async fn main() {
    let mut composition = Composition::new();
    composition.add_pattern(
        "kick".to_string(),
        PatternBuilder::new()
            .config(PatternConfig::new(120, 1.0, 8))
            .sample(load_sample_wav("samples/kick.wav"))
            .trigger_beats(vec![1, 3, 5, 7, 8])
            .effect(Effect::Crunchy(0.9))
            .build()
            .unwrap(),
    );
    composition.add_pattern(
        "hat".to_string(),
        PatternBuilder::new()
            .config(PatternConfig::new(120, 1.0, 8))
            .sample(load_sample_wav("samples/hat.wav"))
            .trigger_beats(vec![1, 2, 3, 4, 5, 6, 7, 8])
            // .effect(Effect::Crunchy(1.1))
            .effect(Effect::Amplify(3.0))
            .build()
            .unwrap(),
    );
    composition.add_pattern(
        "strings".to_string(),
        PatternBuilder::new()
            .config(PatternConfig::new(120, 1.0, 8))
            .sample(load_sample_mp3("samples/strings.mp3"))
            .trigger_beats(vec![1, 2, 4, 5, 6])
            .effect(Effect::Crunchy(0.9))
            .effect(Effect::Amplify(100.0))
            // .effect(Effect::PitchShift(12))
            .build()
            .unwrap(),
    );

    play_composition(&composition);
}
