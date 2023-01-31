use crate::*;
use nnnoiseless::dasp::sample::FromSample;
use std::sync::mpsc;

type Chunk = sized_chunks::Chunk<f32, 32>;

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
    let (tx, rx) = mpsc::channel::<Chunk>();
    let (tx_recycle, rx_recycle) = mpsc::channel::<Chunk>();
    tx_recycle.send(Chunk::new()).unwrap();
    tx_recycle.send(Chunk::new()).unwrap();
    tx_recycle.send(Chunk::new()).unwrap();
    tx_recycle.send(Chunk::new()).unwrap();
    tx_recycle.send(Chunk::new()).unwrap();
    (
        fn_stream(|emitter| async move {
            let tx_err_ptr = tx_err.clone();
            let out_stream = output_device
                .build_output_stream(
                    &config,
                    move |output: &mut [f32], _| {
                        for output_sample in output {
                            let mut chunk = rx.recv().unwrap();
                            *output_sample = chunk.pop_front();
                            tx_recycle.send(chunk).unwrap();
                        }
                    },
                    move |e| tx_err_ptr.send(e.into()).unwrap(),
                )
                .expect("Failed to build internal output stream.");

            out_stream
                .play()
                .expect("Failed to play internal output stream.");

            while let Some(next_input) = input.next().await {
                let mut chunk = rx_recycle.recv().unwrap();
                chunk.push_back(next_input);
                tx.send(chunk).expect("Failed to send on internal MPSC.");
                emitter.emit(next_input).await;
            }
        }),
        rx_err,
    )
}

// pub fn getstream_to_speaker<S>(
//     config: StreamConfig,
//     output_device: Device,
//     mut input: S,
// ) -> (impl Stream<Item = PCMUnit>, Receiver<SpeakerError>)
//     where
//         S: Stream<Item = PCMUnit> + Unpin,
// {
//     let (tx_err, rx_err) = mpsc::channel::<SpeakerError>();
//     let (tx, rx) = mpsc::channel::<f32>();
//     (
//         fn_stream(|emitter| async move {
//             let tx_err_ptr = tx_err.clone();
//             let out_stream = output_device
//                 .build_output_stream(
//                     &config,
//                     move |output: &mut [f32], _| {
//                         for output_sample in output {
//                             *output_sample = rx.recv().unwrap();
//                         }
//                     },
//                     move |e| tx_err_ptr.send(e.into()).unwrap(),
//                 )
//                 .expect("Failed to build internal output stream.");
//
//             out_stream
//                 .play()
//                 .expect("Failed to play internal output stream.");
//
//             while let Some(next_input) = input.next().await {
//                 tx.send(next_input)
//                     .expect("Failed to send on internal MPSC.");
//                 emitter.emit(next_input).await;
//             }
//         }),
//         rx_err,
//     )
// }
