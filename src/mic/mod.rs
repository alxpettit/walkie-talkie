use crate::*;
use async_fn_stream::{fn_stream, try_fn_stream};

use futures::{stream, StreamExt};

use futures::stream::FuturesUnordered;
use nnnoiseless::dasp::Signal;
use std::sync::mpsc::{Receiver, Sender};

fn normalize_stream<S: Stream<Item = Vec<f32>> + Unpin>(
    mut input_stream: S,
) -> impl Stream<Item = [f32; 100]> {
    let mut next_chunk = vec![];
    fn_stream(|emitter| async move {
        while let Some(input_chunk) = input_stream.next().await {
            let input_len = input_chunk.len();
            let mut i = 0;
            while i < input_len {
                next_chunk.extend(input_chunk[i..std::cmp::min(i + 100, input_len)].iter());
                if next_chunk.len() >= 100 {
                    let mut normalized_chunk = [0.0; 100];
                    normalized_chunk.copy_from_slice(&next_chunk[..100]);
                    next_chunk.drain(..100);
                    emitter.emit(normalized_chunk).await;
                }
                i += 100;
            }
        }
    })
}

pub fn getstream_from_mic(
    config: cpal::StreamConfig,
    input_device: cpal::Device,
) -> impl Stream<Item = PCMChunk> {
    fn_stream(|emitter| async move {
        let (tx, rx) = mpsc::channel::<PCMVec>();

        let input_stream = cpal::Device::build_input_stream(
            &input_device,
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                tx.send(data.to_vec()).unwrap();
            },
            move |_err| {},
        )
        .unwrap();

        input_stream.play().unwrap();

        let pre_normalized_mic = fn_stream(|emitter| async move {
            emitter.emit(rx.recv().unwrap()).await;
        });
        pin_mut!(pre_normalized_mic);

        let normalized_mic = normalize_stream(pre_normalized_mic);
        pin_mut!(normalized_mic);

        // TODO: return future directly.

        while let Some(n) = normalized_mic.next().await {
            emitter.emit(n).await;
        }
    })
}
