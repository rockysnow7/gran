use crate::state::{Composition, Pattern};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, BufferSize};
use std::sync::{Arc, Mutex};

pub fn play_composition(composition: &Composition) {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let default_config = device.default_output_config().unwrap();

    let mut stream_config: StreamConfig = default_config.clone().into();
    stream_config.buffer_size = BufferSize::Fixed(256);

    println!("Audio device: {}", device.name().unwrap());
    println!("Audio config: {stream_config:?}");

    let sample_rate = stream_config.sample_rate.0 as usize;

    let patterns = composition.patterns.values().map(|pattern| {
        let mut pattern = pattern.clone();
        pattern.update_sample_rate(sample_rate);
        pattern
    }).collect::<Vec<_>>();

    let err_fn = |err| eprintln!("Audio stream error: {err}");

    let stream = match default_config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &stream_config, patterns, err_fn),
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &stream_config, patterns, err_fn),
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &stream_config, patterns, err_fn),
        _ => panic!("Unsupported sample format"),
    }.unwrap();

    stream.play().unwrap();

    // keep the stream alive
    std::thread::park();
}

fn build_stream<T>(
    device: &Device,
    config: &StreamConfig,
    patterns: Vec<Pattern>,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;
    let patterns = Arc::new(Mutex::new(patterns));

    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let mut patterns_guard = patterns.lock().unwrap();
                let mean = patterns_guard.iter_mut().map(|pattern| pattern.next_sample()).sum::<f32>() / patterns_guard.len() as f32;
                drop(patterns_guard); // Release the lock early
                
                for channel_sample in frame.iter_mut() {
                    *channel_sample = T::from_sample(mean);
                }
            }
        },
        err_fn,
        None,
    )
}
