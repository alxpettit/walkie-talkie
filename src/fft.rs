use crate::*;
use async_fn_stream::fn_stream;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::{Arc, Mutex};

static BUFFER: usize = 1024;

pub fn normalize_buf(buf: &mut Vec<Complex<f32>>) {
    let buf_len = buf.len();
    for x in &mut buf.iter_mut() {
        *x = *x / (buf_len as f32);
    }
}

pub fn real2complex(real: &Vec<f32>) -> Vec<Complex<f32>> {
    real.iter().map(|x| Complex::new(*x, 0.0)).collect()
}

pub fn getstream_fft<S: Stream<Item = PCMUnit> + Unpin>(
    mut input: S,
) -> impl Stream<Item = Complex<f32>> {
    fn_stream(|emitter| async move {
        let mut planner = FftPlanner::new();
        let mut buf: Vec<f32> = Vec::with_capacity(BUFFER);
        loop {
            // Yes, I would have liked to do take().collect(). No, it doesn't work.
            // Borrow checker doesn't like :p
            //while let Some(x) = input.next().await {
            'buf: for _ in 0..BUFFER {
                match input.next().await {
                    Some(x) => buf.push(x),
                    None => {
                        break 'buf;
                    }
                }
            }

            let fft = planner.plan_fft_forward(buf.len());
            let mut complex_buf = real2complex(&buf);
            buf.clear();
            fft.process(&mut complex_buf);
            for item in complex_buf {
                emitter.emit(item).await;
            }
        }
    })
}

pub fn complex_to_real<S: Stream<Item = Complex<f32>> + Unpin>(
    mut input: S,
) -> impl Stream<Item = f32> {
    fn_stream(|emitter| async move {
        while let Some(x) = input.next().await {
            emitter.emit(x.re).await;
        }
    })
}

// mod tests {
//     use super::*;
//     use futures::executor::block_on;
//     use hound::WavSpec;
//     use hound::WavWriter;
//     use std::io::Write;
//     use std::time::{Duration, Instant};
//
//     #[test]
//     fn it_works() -> Result<(), Box<dyn Error>> {
//         let host = cpal::default_host();
//         let input_device = host
//             .default_input_device()
//             .ok_or("No default input device available :c")?;
//         let mut supported_configs_range = input_device.supported_input_configs()?;
//         let supported_config = supported_configs_range
//             .next()
//             .ok_or("Could not get the first supported config from range")?
//             .with_max_sample_rate();
//         let mut config: cpal::StreamConfig = supported_config.into();
//         config.sample_rate = cpal::SampleRate(44_100);
//         config.channels = 2u16;
//
//         let mut tmp = std::env::temp_dir();
//         tmp.push("getstream_from_mic_test.wav");
//
//         let mut wav_writer = WavWriter::create(
//             tmp,
//             WavSpec {
//                 channels: config.channels,
//                 sample_rate: config.sample_rate.0, // Dynamically grab
//                 bits_per_sample: 32,               // Hound locks this at 32
//                 sample_format: hound::SampleFormat::Float,
//             },
//         )?;
//
//         let start = Instant::now();
//         let stream = getstream_from_mic(config, input_device);
//         pin_mut!(stream);
//         while let Some(c) = block_on(stream.next()) {
//             if start.elapsed() > Duration::from_secs(5) {
//                 break;
//             }
//             wav_writer.write_sample(c.unwrap()).unwrap();
//         }
//
//         Ok(())
//     }
// }
