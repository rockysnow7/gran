use crate::sounds::{Grain, SAMPLES_PER_GRAIN, Sound};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig, BufferSize};
use std::sync::{Arc, Mutex, LazyLock};

static HOST: LazyLock<Host> = LazyLock::new(cpal::default_host);
pub static SAMPLE_RATE: LazyLock<usize> = LazyLock::new(|| {
    let device = HOST.default_output_device().unwrap();
    let default_config = device.default_output_config().unwrap();

    default_config.sample_rate().0 as usize
});

pub fn play_sound(sound: &mut dyn Sound) {
    let device = HOST.default_output_device().unwrap();
    let default_config = device.default_output_config().unwrap();

    let mut stream_config: StreamConfig = default_config.clone().into();
    stream_config.buffer_size = BufferSize::Fixed(SAMPLES_PER_GRAIN as u32);

    let err_fn = |err| eprintln!("Audio stream error: {err}");

    let stream = match default_config.sample_format() {
        cpal::SampleFormat::F32 => {
            sound.update_sample_rate(*SAMPLE_RATE);
            build_stream::<f32>(&device, &stream_config, vec![sound.clone_box()], err_fn)
        },
        cpal::SampleFormat::I16 => {
            sound.update_sample_rate(*SAMPLE_RATE);
            build_stream::<i16>(&device, &stream_config, vec![sound.clone_box()], err_fn)
        },
        cpal::SampleFormat::U16 => {
            sound.update_sample_rate(*SAMPLE_RATE);
            build_stream::<u16>(&device, &stream_config, vec![sound.clone_box()], err_fn)
        },
        _ => panic!("Unsupported sample format"),
    }.unwrap();

    stream.play().unwrap();

    // keep the stream alive
    std::thread::park();
}

fn combine_grains(grains: Vec<Grain>) -> Vec<f32> {
    let mut combined = vec![0.0; grains[0].len()];
    for grain in &grains {
        for (i, sample) in grain.iter().enumerate() {
            combined[i] += sample;
        }
    }

    for sample in &mut combined {
        *sample /= grains.len() as f32;
    }

    combined
}

fn build_stream<T>(
    device: &Device,
    config: &StreamConfig,
    sounds: Vec<Box<dyn Sound>>,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;
    let sounds = Arc::new(Mutex::new(sounds));
    let current_grain = Arc::new(Mutex::new(Vec::<f32>::new()));
    let grain_position = Arc::new(Mutex::new(0usize));

    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let mut grain_pos = grain_position.lock().unwrap();
                let mut current_grain_guard = current_grain.lock().unwrap();
                
                // Check if we need to get a new grain
                if current_grain_guard.is_empty() || *grain_pos >= current_grain_guard.len() {
                    let mut sounds_guard = sounds.lock().unwrap();
                    let grains = sounds_guard.iter_mut().map(|sound| sound.next_grain()).collect::<Vec<_>>();
                    *current_grain_guard = combine_grains(grains);
                    *grain_pos = 0;
                    drop(sounds_guard); // Release the lock early
                }
                
                // Get the current sample from the grain
                let sample = if *grain_pos < current_grain_guard.len() {
                    current_grain_guard[*grain_pos]
                } else {
                    0.0 // Silence if we're out of bounds
                };
                
                *grain_pos += 1;
                drop(current_grain_guard);
                drop(grain_pos);

                for channel_sample in frame.iter_mut() {
                    *channel_sample = T::from_sample(sample);
                }
            }
        },
        err_fn,
        None,
    )
}
