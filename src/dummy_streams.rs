use snafu::prelude::*;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::sync::mpsc;
use tokio::sync::broadcast::Sender;
use tracing::{error, trace, Instrument};
use tracing_subscriber;

#[derive(Debug, Snafu)]
enum BroadcastStreamError<T>
where
    T: fmt::Display,
{
    #[snafu(display("Failed to send value: {value}"))]
    FailedToSendValue { value: T },
}

trait SendLogError<T> {
    fn l_send(&mut self, value: T);
}

impl<T> SendLogError<T> for mpsc::Sender<T> {
    fn l_send(self: &mut mpsc::Sender<T>, value: T) {
        match self.send(value) {
            Ok(_) => {
                println!("hmm");
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}

pub async fn repeat<T>(s: Sender<T>, repeat_value: T)
where
    T: Send + Copy + 'static + Debug,
{
    tokio::spawn(async move {
        let r = s.subscribe();
        while r.is_empty() {
            match s.send(repeat_value) {
                Ok(v) => {
                    trace!("Successfully sent value. Receivers: {}", v)
                }
                Err(e) => {
                    error!("Failed to send value. Error: {}", e)
                }
            }
        }
    });
}

// tokio::spawn(async move {
// loop {
// while let Ok(s) = block_on(rx.recv()) {
// println!("{}", s);
// }
// }
// });

mod tests {
    use super::*;
    use futures::executor::block_on;
    use tokio::sync::broadcast;
    use tokio::sync::broadcast::channel;

    #[test_log::test(tokio::test)]
    async fn test_repeat() {
        let (s, mut r) = channel(128);
        repeat(s, 10.0).await;
        assert_eq!(r.recv().await, Ok(10.));
    }
}
