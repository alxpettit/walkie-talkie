use crate::*;

use async_fn_stream::{fn_stream, try_fn_stream};
use snafu::prelude::*;
use std::sync::mpsc::SendError;
use cpal::StreamError;

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


pub fn getstream_to_speaker<S, E>(
    config: StreamConfig,
    output_device: Device,
    mut error_callback: E,
    mut input: S,
) -> impl Stream<Item = PCMUnit>
where
    E: FnMut(SpeakerError) + Send + 'static,
    S: Stream<Item = PCMUnit> + Unpin,
{
    let (tx, rx) = mpsc::channel::<f32>();
    fn_stream(|emitter| async move {
        let out_stream = output_device.build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                for output_sample in output {
                    *output_sample = rx.recv().unwrap();
                }
            },
            move |e| error_callback(e.into()),
        )?;

        out_stream.play()?;

        while let Some(next_input) = input.next().await {
            let inp: f32 = next_input;
            tx.send(inp).or_else(|x| );
            emitter.emit(inp).await;
        }
    })
}
