use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig};
use futures::executor::block_on;
use nnnoiseless::DenoiseState;
use std::error::Error;
use std::hash::Hasher;
use std::ops::Deref;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{sync, thread};
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

struct MicStream {
    tx: Sender<f32>,
    rx: Receiver<f32>,
}

impl MicStream {
    fn new(capacity: usize) -> Self {
        match broadcast::channel::<f32>(capacity) {
            (tx, rx) => Self { tx, rx },
        }
    }

    fn play(&self, config: &StreamConfig, input_device: &Device) -> Result<(), Box<dyn Error>> {
        let sender = self.tx.clone();
        let input_stream = cpal::Device::build_input_stream(
            &input_device,
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for sample in data.to_vec() {
                    self.tx.send(sample.clone()).expect("Error sending");
                }
            },
            move |_err| {},
        )
        .unwrap();
        input_stream.play()?;
        Ok(())
    }
}

fn denoise_stream(mut rx: Receiver<f32>) -> Receiver<f32> {
    let denoise = RwLock::new(DenoiseState::new());
    let (out_tx, mut out_rx) = tokio::sync::broadcast::channel::<f32>(100000);
    let handle = thread::spawn(move || loop {
        let mut frame_output: [f32; DenoiseState::FRAME_SIZE] = MyDefault::default();
        let mut frame_input: [f32; DenoiseState::FRAME_SIZE] = MyDefault::default();
        for s in &mut frame_input {
            *s = block_on(rx.recv()).unwrap() * 32768.0;
        }

        block_on(denoise.write()).process_frame(&mut frame_output, &mut frame_input);
        for s in &frame_output {
            out_tx.send(*s / 32768.0).unwrap();
        }
    });

    out_rx
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

    let mut mic_stream = MicStream::new(100000);
    mic_stream.play(&config, &input_device)?;

    let rx_mic = mic_stream.tx.subscribe();
    let mut denoise_rx = denoise_stream(rx_mic);

    let output_device = host
        .default_output_device()
        .ok_or("No default output device available!")?;
    let out_stream = output_device
        .build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                //  let mut dnoise_ptr_rw = denoise_ptr.blocking_write();
                //    let chunk = rx.recv().iter();
                for output_sample in output {
                    // This had better be zero cost >.>
                    match futures::executor::block_on(denoise_rx.recv()) {
                        Ok(sample) => {
                            //println!("{}", sample);
                            *output_sample = sample;
                        }
                        Err(_) => {}
                    }
                }
            },
            |_| {},
        )
        .expect("TODO: panic message");

    out_stream.play().unwrap();
    loop {
        thread::sleep(Duration::from_secs(10000));
    }
    Ok(())
}
