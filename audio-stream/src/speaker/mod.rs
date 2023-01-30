use crate::*;
use std::sync::mpsc;

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
                .expect("Failed to build internal output stream.");

            out_stream
                .play()
                .expect("Failed to play internal output stream.");

            while let Some(next_input) = input.next().await {
                tx.send(next_input)
                    .expect("Failed to send on internal MPSC.");
                emitter.emit(next_input).await;
            }
        }),
        rx_err,
    )
}

// use crate::*;
// use chonk_chunking::{Chonk, ChonkRemainder};
// use std::sync::mpsc;
//
// #[derive(Debug, Snafu)]
// pub enum SpeakerError {
//     #[snafu(display("StreamError: {}", e))]
//     StreamError { e: cpal::StreamError },
//     #[snafu(display("Error sending stream: {}", e))]
//     StreamSendingError { e: SendError<f32> },
//     #[snafu(display("Build stream error: {}", e))]
//     BuildStreamError { e: BuildStreamError },
//     #[snafu(display("Play stream error: {}", e))]
//     PlayStreamError { e: PlayStreamError },
// }
//
// impl From<StreamError> for SpeakerError {
//     fn from(e: StreamError) -> Self {
//         SpeakerError::StreamError { e }
//     }
// }
// // the trait `From<std::sync::mpsc::SendError<f32>>` is not implemented for `SpeakerError`
//
// impl From<SendError<f32>> for SpeakerError {
//     fn from(e: SendError<f32>) -> Self {
//         SpeakerError::StreamSendingError { e }
//     }
// }
//
// impl From<BuildStreamError> for SpeakerError {
//     fn from(e: BuildStreamError) -> Self {
//         SpeakerError::BuildStreamError { e }
//     }
// }
//
// impl From<PlayStreamError> for SpeakerError {
//     fn from(e: PlayStreamError) -> Self {
//         SpeakerError::PlayStreamError { e }
//     }
// }
// pub fn getstream_to_speaker<S>(
//     config: StreamConfig,
//     output_device: Device,
//     mut input: S,
// ) -> (impl Stream<Item = PCMUnit>, Receiver<SpeakerError>)
// where
//     S: Stream<Item = PCMUnit> + Unpin,
// {
//     let (tx_err, rx_err) = mpsc::channel::<SpeakerError>();
//     let (tx, rx) = mpsc::channel::<Chonk<f32>>();
//     (
//         fn_stream(|emitter| async move {
//             let tx_err_ptr = tx_err.clone();
//             let mut chonk = rx.recv().expect("Hung up :C");
//             let mut remainder: ChonkRemainder<f32> = ChonkRemainder::new();
//             let out_stream = output_device
//                 .build_output_stream(
//                     &config,
//                     move |output: &mut [f32], _| {
//                         remainder = chonk.clone_from(&mut remainder.iter());
//                         chonk.clone_to(&mut output.iter_mut());
//                         // for output_sample in output {
//                         //     *output_sample = rx.recv().unwrap();
//                         // }
//                     },
//                     move |e| tx_err_ptr.send(e.into()).unwrap(),
//                 )
//                 .expect("Failed to build internal output stream.");
//
//             out_stream
//                 .play()
//                 .expect("Failed to play internal output stream.");
//
//             loop {
//                 let mut chonk = Chonk::<f32>::new(1024);
//                 chonk.nom_stream_ref(&mut input).await;
//                 tx.send(chonk).expect("Failed to send in internal MPSC");
//             }
//             // while let Some(next_input) = input.next().await {
//             //     tx.send(next_input)
//             //         .expect("Failed to send on internal MPSC.");
//             //     emitter.emit(next_input).await;
//             // }
//         }),
//         rx_err,
//     )
// }
