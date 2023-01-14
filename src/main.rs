use async_stream::stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig};
use futures::executor::block_on;
use futures::{pin_mut, StreamExt};
use futures_core::Stream;
use nnnoiseless::dasp::Signal;
use nnnoiseless::DenoiseState;
use std::error::Error;
use std::future::Future;
use std::hash::Hasher;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, RwLock};

type Chunk = Vec<f32>;

trait MyDefault {
    fn default() -> Self;
}

impl MyDefault for [f32; DenoiseState::FRAME_SIZE] {
    fn default() -> Self {
        [0.; DenoiseState::FRAME_SIZE]
    }
}

pub fn getstream_mic_input(
    config: cpal::StreamConfig,
    input_device: cpal::Device,
) -> impl Stream<Item = f32> {
    stream! {
        let (tx, rx) = mpsc::channel::<Chunk>();

        let input_stream = cpal::Device::build_input_stream(
            &input_device, &config,  move |data: &[f32], _: &cpal::InputCallbackInfo| {
            tx.send(data.to_vec()).unwrap();
        }, move |_err| {}).expect("Failed to make stream :(");

        input_stream.play().expect("Failed to play stream");

        for data in rx {
            for sample in data {yield sample;}
        }
    }
}

fn getstream_denoise<S: Stream<Item = f32> + Unpin>(mut input: S) -> impl Stream<Item = f32> {
    let denoise = std::sync::RwLock::new(DenoiseState::new());
    let mut frame_output: [f32; DenoiseState::FRAME_SIZE] = MyDefault::default();
    let mut frame_input: [f32; DenoiseState::FRAME_SIZE] = MyDefault::default();
    stream! {
    loop {
            for s in &mut frame_input {
                *s = input.next().await.unwrap() * 32768.0;
            }
            denoise.write().unwrap().process_frame(&mut frame_output, &mut frame_input);
            for s in &frame_output {
                yield *s / 32768.0;
            }
        }
    }
}

fn stream_to_speaker<S: Stream<Item = f32> + Unpin>(
    config: StreamConfig,
    output_device: Device,
    mut input: S,
) -> impl Stream<Item = f32> {
    stream! {
        let (tx, rx) = mpsc::channel::<f32>();
        let out_stream = output_device
        .build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                for output_sample in output {
                    // This had better be zero cost >.>
                    *output_sample = rx.recv().unwrap();
                }
            },
            |_| {},
        )
        .expect("TODO: panic message");

        out_stream.play().unwrap();

        loop {
            match input.next().await {
                Some(sample) => {
                    yield sample;
                    if tx.send(sample).is_err() {
                        println!("Could not send to internal MPSC channel...")
                    }
                }
                None => {
                }
            }
        }
    }
}

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

    let mut mic_stream = getstream_mic_input(config.clone(), input_device);
    pin_mut!(mic_stream);
    let mut denoised_mic_stream = getstream_denoise(mic_stream);
    pin_mut!(denoised_mic_stream);

    let output_device = host
        .default_output_device()
        .ok_or("No default output device available!")?;

    let stream_to_speaker = stream_to_speaker(config, output_device, denoised_mic_stream);
    pin_mut!(stream_to_speaker);
    while let Some(i) = stream_to_speaker.next().await {
        // println!("{}", i);
    }

    // loop {
    //     thread::sleep(Duration::from_secs(100));
    // }
    Ok(())
}
