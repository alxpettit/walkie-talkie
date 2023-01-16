use crate::*;
use async_fn_stream::try_fn_stream;
use std::sync::Mutex;

#[derive(Default)]
pub struct MicStreamFactory {
    input_stream: Option<cpal::Stream>,
}

impl MicStreamFactory {
    pub fn get_stream(
        &mut self,
        config: cpal::StreamConfig,
        input_device: cpal::Device,
    ) -> impl Stream<Item = PCMResult> + '_ {
        try_fn_stream(|emitter| async {
            let emitter = emitter;
            let config = config;
            let input_device = input_device;
            let (tx, rx) = mpsc::channel::<PCMVec>();

            self.input_stream = Some(cpal::Device::build_input_stream(
                &input_device,
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    tx.send(data.to_vec()).unwrap();
                },
                move |_err| {},
            )?);

            for data in rx {
                for sample in data {
                    emitter.emit(sample).await;
                }
            }
            Ok(())
        })
    }

    pub fn play(&mut self) -> Result<(), Box<dyn Error>> {
        self.input_stream
            .as_mut()
            .ok_or("Stream not initialized.")?
            .play()?;
        Ok(())
    }
}
