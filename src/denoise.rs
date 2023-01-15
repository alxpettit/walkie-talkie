use crate::*;

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

pub fn getstream_denoise<S: Stream<Item = PCMResult> + Unpin>(
    mut input: S,
) -> impl Stream<Item = PCMResult> {
    let denoise = std::sync::RwLock::new(DenoiseState::new());
    let mut frame_output: DenoiseChunk = DefaultDenoise::default();
    let mut frame_input: DenoiseChunk = DefaultDenoise::default();
    try_stream! {
        'outer: loop {
            for s in &mut frame_input {
                if let Some(next) = input.next().await {
                    *s = next? * 32768.0;
                } else {
                    break 'outer;
                }
            }
            denoise.write().unwrap().process_frame(&mut frame_output, &mut frame_input);
            for s in &frame_output {
                yield *s / 32768.0;
            }
        }
    }
}
