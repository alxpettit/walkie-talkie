use async_fn_stream::fn_stream;
use futures::Stream;
use futures::StreamExt;

pub async fn getstream_vol_up(
    mult: f32,
    mut input: impl Stream<Item = f32> + Unpin,
) -> impl Stream<Item = f32> {
    fn_stream(|emitter| async move {
        while let Some(x) = input.next().await {
            emitter.emit(x * mult).await;
        }
    })
}
