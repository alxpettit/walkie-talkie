use async_fn_stream::fn_stream;
use chonk_chunking::Chonk;
use futures::Stream;
use futures::StreamExt;

#[deprecated]
pub async fn getstream_vol_scale(
    mult: f32,
    mut input: impl Stream<Item = f32> + Unpin,
) -> impl Stream<Item = f32> {
    fn_stream(|emitter| async move {
        while let Some(x) = input.next().await {
            emitter.emit(x * mult).await;
        }
    })
}

pub async fn getstream_vol_scale_chonk(
    mult: f32,
    mut input: impl Stream<Item = Chonk<f32>> + Unpin,
) -> impl Stream<Item = Chonk<f32>> {
    fn_stream(|emitter| async move {
        while let Some(mut chonk) = input.next().await {
            chonk.for_each_mut(|x| *x *= mult);
            emitter.emit(chonk).await;
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::vol_up::getstream_vol_scale_chonk;
    use async_fn_stream::fn_stream;
    use chonk_chunking::Chonk;
    use futures::executor::block_on;
    use futures::pin_mut;
    use futures::StreamExt;
    use itertools::Itertools;
    use nnnoiseless::dasp::Signal;

    #[test]
    fn basic_vol() {
        let stream = fn_stream(|emitter| async move {
            for _ in 0..10 {
                let new_chonk = Chonk::new_repeat_n(2f32, 100);
                emitter.emit(new_chonk).await;
            }
        });
        pin_mut!(stream);
        let mut stream = block_on(getstream_vol_scale_chonk(2.0, stream));
        pin_mut!(stream);
        while let Some(x) = block_on(stream.next()) {
            assert_eq!(x, Chonk::from(vec![4f32; 100]));
        }
    }
}
