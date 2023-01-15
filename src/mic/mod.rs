use crate::*;
use async_fn_stream::try_fn_stream;
use futures::executor::block_on;
use std::pin::Pin;

pub fn getstream_from_mic(
    config: cpal::StreamConfig,
    input_device: cpal::Device,
) -> impl Stream<Item = PCMResult> {
    try_fn_stream(|emitter| async move {
        let (tx, rx) = mpsc::channel::<f32>();
        //  let mut tmpframe: Frame = [0f32; 1024];

        let input_stream = cpal::Device::build_input_stream(
            &input_device,
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let (chunks, remainder) = data.as_chunks::<1024>();
                for chunk in chunks {
                    for s in chunk {
                        tx.send(*s).unwrap();
                        //*s
                    }
                }
                for s in remainder {
                    tx.send(*s).unwrap();
                }
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
            emitter.emit(data).await;
        }
        Ok(())
    })
}
