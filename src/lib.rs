use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use nnnoiseless::DenoiseState;
use std::error::Error;
use std::ops::Deref;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Sender;

type Chunk = Vec<f32>;

pub fn mic(
    tx: Sender<f32>,
    config: &StreamConfig,
    input_device: &Device,
) -> Result<Stream, Box<dyn Error>> {
    let input_stream = cpal::Device::build_input_stream(
        &input_device,
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for sample in data.to_vec() {
                tx.send(sample).expect("TODO: panic message");
            }
        },
        move |_err| {},
    )?;
    Ok(input_stream)
}
