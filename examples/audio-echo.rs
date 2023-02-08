use crate::mic::getstream_from_mic;
use crate::speaker::getstream_to_speaker;
use audio_stream::denoise::*;
use audio_stream::fft::getstream_fft;
use audio_stream::{denoise, mic, speaker};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig};
use futures::{pin_mut, StreamExt};
use futures_core::Stream;
use next_gen::mk_gen;
use nnnoiseless::DenoiseState;
use std::error::Error;
use std::sync::mpsc;

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
    config.sample_rate = cpal::SampleRate(88_200); // cpal::SampleRate(44_100);

    mk_gen!(let mic_stream = getstream_from_mic(config.clone(), input_device));

    mk_gen!(let denoised_mic_stream = getstream_denoise(mic_stream));

    let output_device = host
        .default_output_device()
        .ok_or("No default output device available!")?;

    // let mic_stream = getstream_fft(mic_stream);
    // pin_mut!(mic_stream);
    // //
    // println!("henlo");
    // while let Some(buf) = complex.next().await {
    //     println!("{:#?}", buf);
    // }

    mk_gen!(let fft_stream = getstream_fft(denoised_mic_stream));

    // mk_gen!(let stream_to_speaker =
    getstream_to_speaker(config, output_device, fft_stream);
    // );
    // for s in stream_to_speaker {
    //     // if let Err(e) = i {
    //     //     println!("{}", e);
    //     // }
    //     //println!("{}", i.unwrap());
    // }
    Ok(())
}
