use crate::pcmtypes::{PCMGenerator, PCMUnit};
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
pub async fn getstream_denoise<'g>(mut input: PCMGenerator<'g>) {
    let denoise = std::sync::RwLock::new(DenoiseState::new());
    let mut frame_output: DenoiseChunk = DefaultDenoise::default();
    let mut frame_input: DenoiseChunk = DefaultDenoise::default();

    'outer: loop {
        for s in &mut frame_input {
            match input.as_mut().resume(()) {
                GeneratorState::Yielded(x) => {
                    *s = x * 32768.0;
                }
                GeneratorState::Returned(_) => break 'outer,
            }
        }
        denoise
            .write()
            .unwrap()
            .process_frame(&mut frame_output, &mut frame_input);

        for s in frame_output {
            yield_!(s / 32768.0);
        }
    }
}
