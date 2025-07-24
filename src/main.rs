#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Pattern, PatternBeat, Saturation, TapeDelay, ADSR}, 
    oscillators::{note, Number, OscillatorBuilder, WaveFunction}, 
    play_sound, 
    sounds::CompositionBuilder, 
};

#[tokio::main]
async fn main() {
    let bass = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(note("C3")),
            amplitude: Number::number(0.7),
            phase: Number::number(0.0),
        })
        .beat_length(1.0)
        .effect(Box::new(ADSR::new(
            0.3,  // attack
            0.2,  // decay
            0.8,  // sustain level
            0.1, // sustain duration
        )))
        .effect(Box::new(Filter::low_pass(
            // dynamic filter for movement
            Number::sine_around(300.0, 150.0, 1.0),
            Number::number(0.6), // resonance
        )))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play { frequency: Some(Number::number(note("G4"))), volume: None },
            PatternBeat::Play { frequency: Some(Number::number(note("A#4"))), volume: None },
            PatternBeat::Play { frequency: Some(Number::number(note("F4"))), volume: None },
            PatternBeat::Play { frequency: Some(Number::number(note("A#4"))), volume: None },
        ])))
        .effect(Box::new(Saturation::new(
            Number::number(4.0),
            Number::number(0.7),
            0.4,
        )))
        .effect(Box::new(TapeDelay::light(0.5)))
        .build();

    // some background noise
    let pink_noise = OscillatorBuilder::new()
        .wave_function(WaveFunction::pink_noise(Number::number(0.001), 10))
        .beat_length(1.0)
        .build();

    // combine everything
    let mut full_track = CompositionBuilder::new()
        .sound(Box::new(pink_noise))
        .sound(Box::new(bass))
        .build();

    play_sound(&mut full_track);
}
