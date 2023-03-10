use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig};
use denoise::DenoiseChunk;
use futures::{pin_mut, StreamExt};
use futures_core::Stream;
use nnnoiseless::DenoiseState;
use pcmtypes::PCMResult;
use std::error::Error;
use std::sync::mpsc;

mod denoise;
mod mic;
mod pcmtypes;
mod speaker;

use pcmtypes::*;

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

    let mic_stream = mic::getstream_from_mic(config.clone(), input_device);
    pin_mut!(mic_stream);

    let denoised_mic_stream = denoise::getstream_denoise(mic_stream);
    pin_mut!(denoised_mic_stream);
    //
    let output_device = host
        .default_output_device()
        .ok_or("No default output device available!")?;

    let (stream_to_speaker, speaker_err) =
        speaker::getstream_to_speaker(config, output_device, denoised_mic_stream);
    pin_mut!(stream_to_speaker);
    while let Some(i) = stream_to_speaker.next().await {}
    Ok(())
}
