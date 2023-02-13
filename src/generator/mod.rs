mod dummy_generator;

use dummy_generator::repeat;
use snafu::prelude::*;
use std::error::Error;
use std::fmt::Debug;
use std::sync::mpsc;
use std::{fmt, thread};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::{SendError, TryRecvError};
use tracing::{error, info, trace, warn, Instrument};

#[derive(Debug, Snafu, Eq, PartialEq)]
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

pub enum GenReq {
    Yield,
    YieldN(usize),
    Stop,
}

/// A wrapper around a tokio broadcast channel, and a MPSC, allowing two way communication
/// between a generator-like function and a consumer function.
pub struct GenSender<T> {
    // a transmitter for broadcasting our generated values
    tx: broadcast::Sender<T>,
    // a receiver for receiving transmissions from consumer functions
    req_rx: mpsc::Receiver<GenReq>,
}

impl<T> GenSender<T> {
    /// Clone transmitter so that we can send broadcasts on this channel, or subscribe more receivers
    fn clone_tx(&self) -> broadcast::Sender<T> {
        self.tx.clone()
    }
    /// Subscribe a receiver from the internal transmitter
    fn subscribe(&self) -> broadcast::Receiver<T> {
        self.tx.subscribe()
    }

    pub(crate) fn req_from_consumer(&self) -> Result<GenReq, mpsc::RecvError> {
        self.req_rx.recv()
    }

    pub(crate) fn send(&self, value: T) -> Result<usize, SendError<T>> {
        self.tx.send(value)
    }
}

struct GenReceiver<T> {
    rx: broadcast::Receiver<T>,
    req_tx: mpsc::Sender<GenReq>,
}

impl<T> GenReceiver<T>
where
    T: Clone,
{
    fn clone_req_tx(&self) -> mpsc::Sender<GenReq> {
        self.req_tx.clone()
    }
    fn req(&self) -> Result<(), mpsc::SendError<GenReq>> {
        self.req_tx.send(GenReq::Yield)
    }
    fn req_n(&self, num: usize) -> Result<(), mpsc::SendError<GenReq>> {
        self.req_tx.send(GenReq::YieldN(num))
    }
    async fn req_recv(&mut self) -> Result<T, Box<dyn Error>> {
        self.req()?;
        Ok(self.recv().await?)
    }
    async fn recv(&mut self) -> Result<T, broadcast::error::RecvError> {
        self.rx.recv().await
    }
    async fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.rx.try_recv()
    }
    async fn stop(&mut self) -> Result<(), mpsc::SendError<GenReq>> {
        self.req_tx.send(GenReq::Stop)
    }
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

    #[test_log::test(tokio::test)]
    async fn test_repeat() {
        let (gen_tx, mut gen_rx) = gen_channel::<f32>(128);
        let tx = gen_tx.clone_tx();

        // We can 'tap' into our receiver
        let rx = tx.subscribe();

        repeat(gen_tx, 10.0);
        // Bulk requests are more efficient, as the generator can chug along at a different rate
        // in its own thread, and fewer background MPSC ops are required
        gen_rx.req_n(3).unwrap();
        // NOTE: If you call this too many times, it will await forever,
        // because the generator is not running enough times :')
        assert_eq!(gen_rx.recv().await, Ok(10.));
        assert_eq!(gen_rx.recv().await, Ok(10.));
        assert_eq!(gen_rx.recv().await, Ok(10.));

        // We get an error if we try to receive another one
        assert!(gen_rx.try_recv().await.is_err());

        // More conventional generator-like behavior.
        // Generator speed is capped at the speed of the consumer,
        // and vice versa. They are tied together.
        assert_eq!(gen_rx.req_recv().await.unwrap(), 10.0);
        assert_eq!(gen_rx.req_recv().await.unwrap(), 10.0);
        assert_eq!(gen_rx.req_recv().await.unwrap(), 10.0);
    }
}
