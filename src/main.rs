#![warn(clippy::all, clippy::pedantic, unused_crate_dependencies)]

use gran::{
    effects::{Filter, Pattern, PatternBeat, Volume, ADSR}, 
    oscillators::{Number, OscillatorBuilder, WaveFunction}, 
    play_sound, 
    sounds::{CompositionBuilder, SampleBuilder}
};

#[tokio::main]
async fn main() {
    // create the drums
    let kick = SampleBuilder::new()
        .samples_from_file("samples/kick.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Volume(Number::number(100.0))))
        .effect(Box::new(Filter::low_pass(
            Number::number(1000.0),
            Number::number(0.5),
        )))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::number(2.5)),
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
        ]).humanize(0.2)))
        .build();

    let hat = SampleBuilder::new()
        .samples_from_file("samples/hat.wav")
        .secs_per_beat(0.25)
        .effect(Box::new(Volume(Number::number(125.0))))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::number(0.8)),
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Play,
        ]).humanize(0.2)))
        .build();

    let drums = CompositionBuilder::new()
        .sound(Box::new(kick))
        .sound(Box::new(hat))
        .build();

    // create the bass synth
    let bass_main = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sawtooth {
            frequency: Number::number(55.0), // A1 note
            amplitude: Number::number(0.7),
            phase: Number::number(0.0),
        })
        .beat_length(0.25) // match the drum beat length
        .effect(Box::new(ADSR::new(
            0.05,  // fast attack
            0.1,   // quick decay
            0.6,   // sustain level
            0.05,  // sustain duration
        )))
        .effect(Box::new(Filter::low_pass(
            // dynamic filter for movement
            Number::sine_around(400.0, 300.0, 0.5), // slow sweep
            Number::number(0.7), // resonance
        )))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::PlayWithVolume(Number::number(0.5)),
            PatternBeat::Play,
            PatternBeat::PlayWithVolume(Number::number(2.5)),
            PatternBeat::PlayWithVolume(Number::number(1.2)),
            PatternBeat::Play,
        ]).humanize(0.1)))
        .build();

    // create the sub bass
    let sub_bass = OscillatorBuilder::new()
        .wave_function(WaveFunction::Sine {
            frequency: Number::number(27.5), // A0
            amplitude: Number::number(0.5),
            phase: Number::number(0.0),
        })
        .beat_length(0.25)
        .effect(Box::new(ADSR::new(0.1, 0.04, 0.7, 0.1)))
        .effect(Box::new(Pattern(vec![
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::Play,
            PatternBeat::Skip,
            PatternBeat::PlayWithVolume(Number::number(0.5)),
            PatternBeat::Play,
            PatternBeat::Play,
            PatternBeat::Skip,
        ])))
        .build();

    let bass = CompositionBuilder::new()
        .sound(Box::new(bass_main))
        .sound(Box::new(sub_bass))
        .effect(Box::new(Volume(Number::number(0.03))))
        .build();

    // some background noise
    let pink_noise = OscillatorBuilder::new()
        .wave_function(WaveFunction::pink_noise(Number::number(0.001), 10))
        .beat_length(1.0)
        .build();

    // combine everything
    let mut full_track = CompositionBuilder::new()
        .sound(Box::new(drums))
        .sound(Box::new(bass))
        .sound(Box::new(pink_noise))
        // master effects
        .effect(Box::new(Volume(Number::number(0.8)))) // overall volume
        .build();

    play_sound(&mut full_track);
}
