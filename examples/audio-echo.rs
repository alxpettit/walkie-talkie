use audio_stream::{mic, speaker};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig};
use crossbeam_channel::bounded;
use nnnoiseless::DenoiseState;
use std::error::Error;
use std::ops::Deref;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Sender;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .ok_or("No default input device available :c")?;
    let mut supported_configs_range = input_device.supported_input_configs()?;
    let supported_config = supported_configs_range
        .next()
        .ok_or("Could not get the first supported config from range")?
        .with_max_sample_rate();
    let mut config: cpal::StreamConfig = supported_config.into();
    config.sample_rate = cpal::SampleRate(44_100);

    //let (tx, mut rx) = broadcast::channel::<f32>(10000);
    let (s, r) = bounded(256);

    let mic_stream = mic(s.clone(), &config, &input_device)?;

    mic_stream.play()?;

    let output_device = host
        .default_output_device()
        .ok_or("No default output device available!")?;
    let out_stream = speaker(r.clone(), &config, &output_device);
    out_stream.play().unwrap();
    loop {
        thread::sleep(Duration::from_secs(10000));
    }
    Ok(())
}
