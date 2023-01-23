use crate::*;

use async_fn_stream::{fn_stream, try_fn_stream};
use cpal::StreamError;
use snafu::prelude::*;
use std::sync::mpsc::SendError;

#[derive(Debug, Snafu)]
pub enum SpeakerError {
    #[snafu(display("StreamError: {}", e))]
    StreamError { e: cpal::StreamError },
    #[snafu(display("Error sending stream: {}", e))]
    StreamSendingError { e: SendError<f32> },
}

impl From<StreamError> for SpeakerError {
    fn from(e: StreamError) -> Self {
        SpeakerError::StreamError { e }
    }
}

pub fn getstream_to_speaker<S>(
    config: StreamConfig,
    output_device: Device,
    mut input: S,
) -> impl Stream<Item = PCMUnit>
where
    S: Stream<Item = PCMResult> + Unpin,
{
    let (tx, rx) = mpsc::channel::<f32>();
    fn_stream(|emitter| async move {
        let out_stream = output_device
            .build_output_stream(
                &config,
                move |output: &mut [f32], _| {
                    for output_sample in output {
                        *output_sample = rx.recv().unwrap();
                    }
                },
                |_| {},
            )
            .unwrap();

        out_stream.play().unwrap();

        while let Some(next_input) = input.next().await {
            let inp: f32 = next_input.unwrap();
            tx.send(inp).unwrap();
            emitter.emit(inp).await;
        }
    })
}
