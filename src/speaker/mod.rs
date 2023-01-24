use crate::*;

use async_fn_stream::{fn_stream, try_fn_stream};
use cpal::{BuildStreamError, PlayStreamError, StreamError};
use snafu::prelude::*;
use std::sync::mpsc::{Receiver, SendError};

#[derive(Debug, Snafu)]
pub enum SpeakerError {
    #[snafu(display("StreamError: {}", e))]
    StreamError { e: cpal::StreamError },
    #[snafu(display("Error sending stream: {}", e))]
    StreamSendingError { e: SendError<f32> },
    #[snafu(display("Build stream error: {}", e))]
    BuildStreamError { e: BuildStreamError },
    #[snafu(display("Play stream error: {}", e))]
    PlayStreamError { e: PlayStreamError },
}

impl From<StreamError> for SpeakerError {
    fn from(e: StreamError) -> Self {
        SpeakerError::StreamError { e }
    }
}
// the trait `From<std::sync::mpsc::SendError<f32>>` is not implemented for `SpeakerError`

impl From<SendError<f32>> for SpeakerError {
    fn from(e: SendError<f32>) -> Self {
        SpeakerError::StreamSendingError { e }
    }
}

impl From<BuildStreamError> for SpeakerError {
    fn from(e: BuildStreamError) -> Self {
        SpeakerError::BuildStreamError { e }
    }
}

impl From<PlayStreamError> for SpeakerError {
    fn from(e: PlayStreamError) -> Self {
        SpeakerError::PlayStreamError { e }
    }
}
pub fn getstream_to_speaker<S>(
    config: StreamConfig,
    output_device: Device,
    mut input: S,
) -> (impl Stream<Item = PCMUnit>, Receiver<SpeakerError>)
where
    S: Stream<Item = PCMUnit> + Unpin,
{
    let (tx_err, rx_err) = mpsc::channel::<SpeakerError>();
    let (tx, rx) = mpsc::channel::<f32>();
    (
        fn_stream(|emitter| async move {
            let tx_err_ptr = tx_err.clone();
            let out_stream = output_device
                .build_output_stream(
                    &config,
                    move |output: &mut [f32], _| {
                        for output_sample in output {
                            *output_sample = rx.recv().unwrap();
                        }
                    },
                    move |e| tx_err_ptr.send(e.into()).unwrap(),
                )
                .unwrap();

            out_stream.play().unwrap();

            while let Some(next_input) = input.next().await {
                tx.send(next_input).unwrap();
                emitter.emit(next_input).await;
            }
        }),
        rx_err,
    )
}

mod tests {
    use super::*;
    use futures::executor::block_on;
    use hound::WavReader;
    use hound::WavSpec;
    use hound::WavWriter;
    use std::io::Write;
    use std::process::exit;
    use std::time::{Duration, Instant};

    #[test]
    fn speaker() -> Result<(), Box<dyn Error>> {
        let host = cpal::default_host();
        let output_device = host
            .default_output_device()
            .ok_or("No default input device available :c")?;
        let mut supported_configs_range = output_device.supported_input_configs()?;
        let supported_config = supported_configs_range
            .next()
            .ok_or("Could not get the first supported config from range")?
            .with_max_sample_rate();
        let mut config: cpal::StreamConfig = supported_config.into();
        config.sample_rate = cpal::SampleRate(44_100);
        config.channels = 2u16;

        let mut wav = WavReader::open("assets/squee.wav").unwrap();
        let input_stream = fn_stream(|emitter| async move {
            // let (tx, rx) = mpsc::channel::<Vec<f32>>();
            // let stream = output_device
            //     .build_output_stream(
            //         &config.clone(),
            //         move |data: &mut [f32], info| {
            //             tx.send(data.to_vec()).unwrap();
            //         },
            //         |_| {},
            //     )
            //     .unwrap();
            //
            // stream.play().unwrap();

            for sample in wav.samples::<f32>() {
                println!("{:#?}", sample);
                let sample = sample.unwrap();
                block_on(emitter.emit(sample));
            }
        });
        pin_mut!(input_stream);
        let (output_stream, e) = getstream_to_speaker(config.clone(), output_device, input_stream);
        pin_mut!(output_stream);
        while let Some(_) = block_on(output_stream.next()) {}

        Ok(())
    }
}
