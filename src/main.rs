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
            0.2,  // attack
            0.1,  // decay
            0.6,  // sustain level
            0.05, // sustain duration
        )))
        .effect(Box::new(Filter::low_pass(
            // dynamic filter for movement
            Number::sine_around(300.0, 200.0, 2.0), // slow sweep
            Number::number(0.6), // resonance
        )))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play { frequency: Some(Number::number(note("A2"))), volume: None },
            PatternBeat::Play { frequency: Some(Number::number(note("A3"))), volume: None },
            PatternBeat::Play { frequency: Some(Number::number(note("C4"))), volume: None },
            PatternBeat::Play { frequency: Some(Number::number(note("G3"))), volume: None },
        ]).humanize(0.5)))
        .effect(Box::new(Saturation::new(
            Number::number(5.0),
            Number::number(0.5),
            0.6,
        )))
        .effect(Box::new(TapeDelay::new(
            0.3,
            Number::number(0.2),
            Number::number(0.1),
            0.001,
            0.1,
            0.0005,
            1.0,
        )))
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
