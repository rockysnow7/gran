#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{play_composition, Sound, Composition, Sample, Gain};
use rodio::{Decoder, Source};
use std::{fs::File, io::BufReader};

// returns (samples, sample rate)
fn load_sample_wav(path: &str) -> (Vec<f32>, usize) {
    let mut reader = hound::WavReader::open(path).unwrap();
    let sample_rate = reader.spec().sample_rate;
    let samples: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();

    (samples.iter().map(|s| *s as f32 / i32::MAX as f32).collect(), sample_rate as usize)
}

// returns (samples, sample rate)
fn load_sample_mp3(path: &str) -> (Vec<f32>, usize) {
    let file = File::open(path).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();
    let sample_rate = source.sample_rate();

    let samples: Vec<f32> = source
        .into_iter()
        .map(|sample| sample as f32 / i16::MAX as f32)
        .collect();
    
    (samples, sample_rate as usize)
}

#[tokio::main]
async fn main() {
    let (samples, sample_rate) = load_sample_wav("samples/kick.wav");
    let mut kick = Sample::new(samples, sample_rate, 1.0);
    kick.add_effect(Box::new(Gain(100.0)));

    let (samples, sample_rate) = load_sample_wav("samples/hat.wav");
    let mut hat = Sample::new(samples, sample_rate, 0.5);
    hat.add_effect(Box::new(Gain(100.0)));

    let mut composition = Composition::new();
    composition.add_sound("kick".to_string(), Box::new(kick));
    composition.add_sound("hat".to_string(), Box::new(hat));

    play_composition(&composition);
}
