use crate::*;
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

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

pub fn getstream_to_speaker<S, E>(
    config: StreamConfig,
    output_device: Device,
    mut input: S,
) -> (impl Stream<Item = PCMUnit>, Receiver<SpeakerError>)
where
    E: FnMut(SpeakerError) + Send + 'static,
    S: Stream<Item = PCMUnit> + Unpin,
{
    let (tx, rx) = mpsc::channel::<f32>();
    let (tx_err, rx_err) = mpsc::channel::<SpeakerError>();
    (
        fn_stream(move |emitter| async move {
            let tx_error_ptr = tx_err.clone();
            let out_stream = output_device.build_output_stream(
                &config,
                move |output: &mut [f32], _| {
                    for output_sample in output {
                        *output_sample = rx.recv().unwrap();
                    }
                },
                move |e| tx_error_ptr.send(e.into()).unwrap(),
            );

            let tx_error_ptr2 = tx_err.clone();
            match out_stream {
                Err(e) => {
                    tx_error_ptr2.send(e.into()).unwrap();
                }
                Ok(stream) => {
                    let play_status = stream.play();
                    if let Err(e) = play_status {
                        tx_error_ptr2.send(e.into()).unwrap()
                    }
                }
            }

            let tx_error_ptr3 = tx_err.clone();
            while let Some(next_input) = input.next().await {
                let inp: f32 = next_input;
                tx.send(inp)
                    .unwrap_or_else(|e| tx_error_ptr3.send(e.into()).unwrap());
                emitter.emit(inp).await;
            }
        }),
        rx_err,
    )
}
