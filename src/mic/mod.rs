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

mod tests {
    use super::*;
    use futures::executor::block_on;
    use hound::WavSpec;
    use hound::WavWriter;
    use std::io::Write;
    use std::time::{Duration, Instant};

    #[test]
    fn it_works() -> Result<(), Box<dyn Error>> {
        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .ok_or("No default input device available :c")?;
        let mut supported_configs_range = input_device.supported_input_configs()?;
        let supported_config = supported_configs_range
            .next()
            .ok_or("Could not get the first supported config from range")?
            .with_max_sample_rate();
        let mut config: cpal::StreamConfig = supported_config.into();
        config.sample_rate = cpal::SampleRate(44_100);
        config.channels = 2u16;

        let mut tmp = std::env::temp_dir();
        tmp.push("getstream_from_mic_test.wav");

        let mut wav_writer = WavWriter::create(
            tmp,
            WavSpec {
                channels: config.channels,
                sample_rate: config.sample_rate.0, // Dynamically grab
                bits_per_sample: 32,               // Hound locks this at 32
                sample_format: hound::SampleFormat::Float,
            },
        )?;

        let start = Instant::now();
        let stream = getstream_from_mic(config, input_device);
        pin_mut!(stream);
        while let Some(c) = block_on(stream.next()) {
            if start.elapsed() > Duration::from_secs(5) {
                break;
            }
            wav_writer.write_sample(chunk.unwrap()).unwrap();
        }

        Ok(())
    }
}
