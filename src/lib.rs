use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use crossbeam_channel::{bounded, Receiver, Sender};
use nnnoiseless::DenoiseState;
use std::error::Error;
use std::ops::Deref;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

pub fn mic(
    tx: Sender<f32>,
    config: &StreamConfig,
    input_device: &Device,
) -> Result<Stream, Box<dyn Error>> {
    //let (s_err, r_err) = bounded(128);
    let input_stream = cpal::Device::build_input_stream(
        &input_device,
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for sample in data {
                tx.send(*sample);
                // match tx.send(*sample) {
                //     Err(e) => {
                //         s_err.send(Box::new(e));
                //         s_err.send_deadline()
                //     }
                // }
            }
        },
        move |_err| {},
    )?;
    Ok(input_stream)
}

pub fn speaker(rx: Receiver<f32>, config: &StreamConfig, output_device: &Device) -> Stream {
    output_device
        .build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                for output_sample in output {
                    // This had better be zero cost >.>
                    match rx.recv() {
                        Ok(sample) => {
                            *output_sample = sample;
                        }
                        Err(_) => {}
                    }
                }
            },
            |_| {},
        )
        .expect("TODO: panic message")
}
