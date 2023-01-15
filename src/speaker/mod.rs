use crate::*;

use async_fn_stream::try_fn_stream;

pub fn getstream_to_speaker<S: Stream<Item = PCMResult> + Unpin>(
    config: StreamConfig,
    output_device: Device,
    mut input: S,
) -> impl Stream<Item = PCMResult> {
    let (tx, rx) = mpsc::channel::<f32>();
    try_fn_stream(|emitter| async move {
        let out_stream = output_device.build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                for output_sample in output {
                    *output_sample = rx.recv().unwrap();
                }
            },
            |_| {},
        )?;

        out_stream.play()?;

        while let Some(next_input) = input.next().await {
            let inp: f32 = next_input?;
            tx.send(inp)?;
            emitter.emit(inp).await;
        }
        Ok(())
    })
}
