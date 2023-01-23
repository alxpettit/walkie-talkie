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
    S: Stream<Item = PCMChunk> + Unpin,
{
    let (tx_err, rx_err) = mpsc::channel::<SpeakerError>();
    let (tx, rx) = mpsc::channel::<PCMChunk>();
    (
        fn_stream(|emitter| async move {
            let tx_err_ptr = tx_err.clone();
            let out_stream = output_device
                .build_output_stream(
                    &config,
                    move |output: &mut [f32], _| {
                        let mut output_index = 0;
                        let output_chunk_size = output.len();

                        while let Ok(mut chunk) = rx.recv() {
                            for v in &mut chunk {
                                output[output_index] = *v;
                            }
                        }
                        // for output_sample in output {
                        //     *output_sample = rx.recv().unwrap();
                        // }
                    },
                    move |e| tx_err_ptr.send(e.into()).unwrap(),
                )
                .unwrap();

            out_stream.play().unwrap();

            while let Some(next_input) = input.next().await {
                tx.send(next_input).unwrap();
                //emitter.emit(buf).await;
            }
        }),
        rx_err,
    )
}
