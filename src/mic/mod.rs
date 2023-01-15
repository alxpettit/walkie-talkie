use crate::*;
use async_fn_stream::try_fn_stream;
use futures::executor::block_on;
use std::pin::Pin;

pub fn getstream_from_mic(
    config: cpal::StreamConfig,
    input_device: cpal::Device,
) -> impl Stream<Item = PCMResult> {
    try_fn_stream(|emitter| async move {
        let (tx, rx) = mpsc::channel::<Frame>();
        //  let mut tmpframe: Frame = [0f32; 1024];

        let input_stream = cpal::Device::build_input_stream(
            &input_device,
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let (chunks, remainder) = data.as_chunks::<8>();
                for chunk in chunks {
                    tx.send(*chunk).unwrap();
                }
                let mut final_frame = [0f32; 8];
                for (i, s) in remainder.iter().enumerate() {
                    final_frame[i] = *s;
                }
                tx.send(final_frame).unwrap();
                // tmpframe = data.as_chunks();

                // for s in data {
                //     block_on(emitter.emit(s));
                // }

                // tx.send(Pin::new(&data)).unwrap();
            },
            move |_err| {},
        )?;

        input_stream.play()?;

        for data in rx {
            for s in data {
                emitter.emit(s).await;
            }
        }
        Ok(())
    })
}
