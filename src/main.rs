#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Pattern, PatternBeat, Saturation, Volume, ADSR}, 
    oscillators::{Number, OscillatorBuilder, WaveFunction}, 
    play_sound, 
    sounds::{CompositionBuilder, SampleBuilder}, 
};

#[tokio::main]
async fn main() {
    let bass_1 = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(55.0), // A1 note
            amplitude: Number::number(0.7),
            phase: Number::number(0.0),
        })
        .beat_length(1.0)
        .effect(Box::new(ADSR::new(
            0.05,  // fast attack
            0.1,   // quick decay
            0.6,   // sustain level
            0.05,  // sustain duration
        )))
        .effect(Box::new(Filter::low_pass(
            // dynamic filter for movement
            Number::sine_around(300.0, 200.0, 2.0), // slow sweep
            Number::number(0.8), // resonance
        )))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
        ])))
        .build();

    let bass_2 = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(55.0), // A1 note
            amplitude: Number::number(0.7),
            phase: Number::number(0.0),
        })
        .beat_length(1.0)
        .effect(Box::new(ADSR::new(
            0.05,  // fast attack
            0.1,   // quick decay
            0.6,   // sustain level
            0.05,  // sustain duration
        )))
        .effect(Box::new(Filter::low_pass(
            // dynamic filter for movement
            Number::sine_around(300.0, 200.0, 2.0), // slow sweep
            Number::number(0.8), // resonance
        )))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Skip,
            PatternBeat::Play,
        ])))
        .effect(Box::new(Saturation::new(
            Number::sine_around(6.0, 4.0, 1.0),
            Number::number(1.0),
            0.6,
        )))
        .build();

    let bass_composition = CompositionBuilder::new()
        .sound(Box::new(bass_1))
        .sound(Box::new(bass_2))
        .build();

    // drums
    let kick = SampleBuilder::new()
        .samples_from_file("samples/kick.wav")
        .secs_per_beat(0.5)
        .effect(Box::new(Volume(Number::number(350.0))))
        .effect(Box::new(Saturation::new(Number::number(10.0), Number::number(0.5), 0.8)))
        .build();

    let drums_composition = CompositionBuilder::new()
        .sound(Box::new(kick))
        .build();

    // some background noise
    let pink_noise = OscillatorBuilder::new()
        .wave_function(WaveFunction::pink_noise(Number::number(0.001), 10))
        .beat_length(1.0)
        .build();

    // combine everything
    let mut full_track = CompositionBuilder::new()
        .sound(Box::new(pink_noise))
        .sound(Box::new(bass_composition))
        // .sound(Box::new(drums_composition))
        // master effects
        .effect(Box::new(Volume(Number::number(0.8)))) // overall volume
        .build();

    play_sound(&mut full_track);
}
