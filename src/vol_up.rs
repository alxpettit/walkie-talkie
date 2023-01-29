use crate::chonk::Chonk;
use async_fn_stream::fn_stream;
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
