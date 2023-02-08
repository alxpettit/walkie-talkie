use crate::*;
use std::pin::Pin;
use std::sync::mpsc;

use crate::pcmtypes::{PCMGenerator, PCMUnit};
use async_fn_stream::{fn_stream, try_fn_stream};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BuildStreamError, Device, PlayStreamError, StreamConfig, StreamError};
use futures::StreamExt;
use futures_core::Stream;
use next_gen::prelude::*;
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

#[generator(yield(PCMUnit))]
pub fn getstream_to_speaker<'g>(
    config: StreamConfig,
    output_device: Device,
    input: PCMGenerator<'g>,
) {
    let (tx_err, rx_err) = mpsc::channel::<SpeakerError>();
    let (tx, rx) = mpsc::channel::<f32>();
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
        .expect("Failed to build internal output stream.");

    out_stream
        .play()
        .expect("Failed to play internal output stream.");

    for next_input in input {
        tx.send(next_input)
            .expect("Failed to send on internal MPSC.");
        yield_!(next_input);
    }
}
