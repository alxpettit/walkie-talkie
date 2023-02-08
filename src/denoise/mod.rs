use crate::pcmtypes::PCMUnit;
use crate::*;
use async_fn_stream::{fn_stream, try_fn_stream};
use futures::StreamExt;
use futures_core::Stream;
use next_gen::prelude::*;
use nnnoiseless::DenoiseState;
use wide::f32x8;

// The chunk type & size expected by the nnnoiseless library
pub type DenoiseChunk = [f32; DenoiseState::FRAME_SIZE];

pub trait DefaultDenoise {
    fn default() -> Self;
}

impl DefaultDenoise for [f32; DenoiseState::FRAME_SIZE] {
    fn default() -> Self {
        [0.; DenoiseState::FRAME_SIZE]
    }
}

#[generator(yield(PCMUnit))]
pub async fn getstream_denoise<
    S: Pin<Generator<Yield = PCMUnit, Return = ()>> + Unpin + futures::Stream,
>(
    mut input: S,
) -> impl Stream<Item = PCMUnit> {
    let denoise = std::sync::RwLock::new(DenoiseState::new());
    let mut frame_output: DenoiseChunk = DefaultDenoise::default();
    let mut frame_input: DenoiseChunk = DefaultDenoise::default();
    // fn_stream(|emitter| async move {
    //     'outer: loop {
    for s in &mut frame_input {
        let mut next = || input.as_mut().resume(());
        if let Some(next) = next() {
            *s = next * 32768.0;
        } else {
            break;
        }
    }
    denoise
        .write()
        .unwrap()
        .process_frame(&mut frame_output, &mut frame_input);

    let scaled: Vec<[f32; 8]> = frame_output
        .chunks(8)
        .map(|chunk| {
            let scaled = f32x8::from(chunk) / 32768.0;
            scaled.into()
        })
        .collect();

    for chunk in scaled {
        for s in &chunk {
            yield_!(s);
            //emitter.emit(*s).await;
        }
    }
    //    }
    // })
}
