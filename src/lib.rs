use async_trait::async_trait;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use futures::executor::block_on;
use futures::SinkExt;
use log::{error, trace};
use nnnoiseless::DenoiseState;
use std::error::Error;
use std::ops::Deref;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::{RecvError, SendError};
use tokio::sync::broadcast::{channel, Receiver, Sender};

trait SendLogError<T> {
    fn l_send(&mut self, value: T);
}

impl<T> SendLogError<T> for mpsc::Sender<T> {
    fn l_send(self: &mut mpsc::Sender<T>, value: T) {
        match self.send(value) {
            Ok(_) => {}
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}

pub fn mic(
    tx: Sender<f32>,
    config: &StreamConfig,
    input_device: &Device,
) -> Result<Stream, Box<dyn Error>> {
    let (s_err, r_err) = mpsc::channel::<Box<dyn Error>>();
    trace!("Begin building input stream");
    Ok(Device::build_input_stream(
        &input_device,
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for sample in data {
                if let Err(e) = tx.send(*sample) {}
            }
        },
        move |e| {
            error!(
                "CPAL stream error callback triggered for input stream: {}",
                e
            );
        },
    )?)
}

pub fn speaker(
    mut rx: Receiver<f32>,
    config: &StreamConfig,
    output_device: &Device,
) -> Result<Stream, Box<dyn Error>> {
    trace!("Begin building output stream");
    Ok(output_device.build_output_stream(
        &config,
        move |output: &mut [f32], _| {
            for output_sample in output {
                // This had better be zero cost >.>
                match futures::executor::block_on(rx.recv()) {
                    Ok(sample) => {
                        *output_sample = sample;
                    }
                    Err(e) => {
                        error!(
                            "CPAL stream error callback triggered for output stream: {}",
                            e
                        );
                    }
                }
            }
        },
        |_| {},
    )?)
}

pub async fn print_broadcast(mut rx: Receiver<f32>) {
    tokio::spawn(async move {
        loop {
            while let Ok(s) = block_on(rx.recv()) {
                println!("{}", s);
            }
        }
    });
}
