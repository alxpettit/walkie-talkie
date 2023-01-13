use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use nnnoiseless::DenoiseState;
use std::error::Error;
use std::ops::Deref;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

type Chunk = Vec<f32>;
fn main() -> Result<(), Box<dyn Error>> {
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

    let (tx, rx) = mpsc::channel::<f32>();

    let input_stream = cpal::Device::build_input_stream(
        &input_device,
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for s in data.to_vec() {
                tx.send(s).unwrap();
            }
        },
        move |_err| {},
    )?;
    input_stream.play().expect("TODO: panic message");

    let output_device = host
        .default_output_device()
        .ok_or("No default output device available!")?;
    let out_stream = output_device
        .build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                let chunk = rx.recv().unwrap();
                for sample in output {
                    *sample = rx.recv().unwrap();
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
