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

struct Mic<T> {
    senders: Vec<Sender<T>>,
}

impl<T> Default for Mic<T> {
    fn default() -> Self {
        Self {
            senders: Vec::new(),
        }
    }
}

impl<T> Mic<T> {
    pub fn new() -> Self {
        Mic::default()
    }

    pub fn mk_stream(
        tx: Sender<f32>,
        config: &StreamConfig,
        input_device: &Device,
    ) -> Result<(Stream, Receiver<Box<dyn Error>>), Box<dyn Error>> {
        let (s_err, r_err) = bounded(128);
        let input_stream = cpal::Device::build_input_stream(
            &input_device,
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for sample in data {
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
        Ok((input_stream, r_err))
    }
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
