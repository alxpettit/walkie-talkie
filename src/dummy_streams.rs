use event_listener::{Event, EventListener};
use snafu::prelude::*;
use std::error::Error;
use std::fmt::Debug;
use std::sync::{mpsc, Arc};
use std::{fmt, thread};
use tokio::sync::broadcast;
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

pub fn repeat<T>(s: Sender<T>, req_event: mpsc::Receiver<usize>, repeat_value: T)
where
    T: Send + Copy + 'static + Debug,
{
    thread::spawn(move || {
        let mut requested = 0usize;
        loop {
            match req_event.recv() {
                Ok(new_requested) => {
                    requested = new_requested;
                }
                Err(e) => {
                    warn!("Receiver hung up: {}", e);
                }
            }
            for _ in 0..requested {
                match s.send(repeat_value) {
                    Ok(v) => {
                        trace!("Successfully sent value. Receivers: {}", v)
                    }
                    Err(e) => {
                        warn!("Failed to send value. Error: {}", e)
                    }
                }
            }
        }
    });
}

enum GenReq {
    Yield,
    YieldN(usize),
    Stop,
}

struct GenSender<T> {
    tx: broadcast::Sender<T>,
    req_rx: mpsc::Receiver<GenReq>,
}

struct GenReceiver<T> {
    rx: broadcast::Receiver<T>,
    req_tx: mpsc::Sender<GenReq>,
}

fn gen_channel<T>(capacity: usize) -> (GenSender<T>, GenReceiver<T>)
where
    T: Clone,
{
    let (tx, mut rx) = broadcast::channel::<T>(capacity);
    let (req_tx, req_rx) = mpsc::channel::<GenReq>();

    (GenSender { tx, req_rx }, GenReceiver { rx, req_tx })
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
        event_tx.send(3).unwrap();
        assert_eq!(r.recv().await, Ok(10.));
        assert_eq!(r.recv().await, Ok(10.));
        assert_eq!(r.recv().await, Ok(10.));
    }
}
