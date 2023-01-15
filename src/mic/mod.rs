use crate::*;
use async_fn_stream::try_fn_stream;

pub fn getstream_from_mic(
    config: cpal::StreamConfig,
    input_device: cpal::Device,
) -> impl Stream<Item = PCMResult> {
    try_fn_stream(|emitter| async move {
        let (tx, rx) = mpsc::channel::<PCMVec>();

        let input_stream = cpal::Device::build_input_stream(
            &input_device,
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                tx.send(data.to_vec()).unwrap();
            },
            move |_err| {},
        )?;

        input_stream.play()?;

        for data in rx {
            for sample in data {
                emitter.emit(sample).await;
            }
        }
        Ok(())
    })
}
