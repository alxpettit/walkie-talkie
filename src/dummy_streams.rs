use event_listener::{Event, EventListener};
use snafu::prelude::*;
use std::error::Error;
use std::fmt::Debug;
use std::sync::{mpsc, Arc};
use std::{fmt, thread};
use tokio::sync::broadcast::Sender;
use tracing::{error, info, trace, warn, Instrument};
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

pub fn repeat<T>(s: Sender<T>, req_event: mpsc::Receiver<()>, repeat_value: T)
where
    T: Send + Copy + 'static + Debug,
{
    thread::spawn(move || loop {
        match req_event.recv() {
            Ok(_) => {}
            Err(e) => {
                info!("Receiver hung up");
            }
        }
        match s.send(repeat_value) {
            Ok(v) => {
                trace!("Successfully sent value. Receivers: {}", v)
            }
            Err(e) => {
                warn!("Failed to send value. Error: {}", e)
            }
        }
    });
}

mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use tokio::sync::broadcast;
    use tokio::sync::broadcast::channel;

    #[test_log::test(tokio::test)]
    async fn test_repeat() {
        let (s, mut r) = channel(128);
        let (event_tx, event_rx) = mpsc::channel();
        //let request: Arc<AtomicUsize> = Arc::new(AtomicUsize::default());
        repeat(s.clone(), event_rx, 10.0);
        event_tx.send(()).unwrap();
        event_tx.send(()).unwrap();
        event_tx.send(()).unwrap();
        assert_eq!(r.recv().await, Ok(10.));
        assert_eq!(r.recv().await, Ok(10.));
        assert_eq!(r.recv().await, Ok(10.));
    }
}
