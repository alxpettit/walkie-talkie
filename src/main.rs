use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig, SupportedStreamConfigsError};
use denoise::DenoiseChunk;
use futures::executor::block_on;
use futures::{pin_mut, FutureExt, StreamExt};
use futures_core::Stream;
use nnnoiseless::DenoiseState;
use pcmtypes::PCMResult;
use snafu::prelude::*;
use std::error::Error;
use std::sync::mpsc;

mod denoise;
mod fft;
mod mic;
mod pcmtypes;
mod speaker;
mod vol_up;
use crate::fft::{getstream_complex_to_real, getstream_fft};
use crate::vol_up::getstream_vol_up;
use pcmtypes::*;

async fn audio_thread() -> Result<(), Box<dyn Error>> {
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
    config.sample_rate = cpal::SampleRate(88_200); // cpal::SampleRate(44_100);

    let output_device = host
        .default_output_device()
        .ok_or("No default output device available!")?;

    let mic_stream = mic::getstream_from_mic(config.clone(), input_device);
    pin_mut!(mic_stream);

    let mic_stream = getstream_vol_up(40., mic_stream).await;
    pin_mut!(mic_stream);

    let mic_stream = denoise::getstream_denoise(mic_stream);
    pin_mut!(mic_stream);

    let mic_stream = getstream_vol_up(2., mic_stream).await;
    pin_mut!(mic_stream);

    let mic_stream = getstream_fft(mic_stream);
    pin_mut!(mic_stream);

    //
    // println!("henlo");
    // while let Some(buf) = complex.next().await {
    //     println!("{:#?}", buf);
    // }

    let (stream_to_speaker, _) = speaker::getstream_to_speaker(config, output_device, mic_stream);
    pin_mut!(stream_to_speaker);
    while let Some(i) = stream_to_speaker.next().await {
        // if let Err(e) = i {
        //     println!("{}", e);
        // }
        //println!("{}", i.unwrap());
    }
    Ok(())
}
async fn audio_thread_noerror() {
    audio_thread().await.unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tokio::spawn(async move {
        block_on(audio_thread_noerror());
    });
    // let thread = audio_thread();
    // let t = thread.remote_handle().0;
    loop {}
    Ok(())
}
