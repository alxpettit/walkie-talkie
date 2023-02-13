use crate::generator::{GenReq, GenSender};
use std::fmt::Debug;
use std::thread;
use tracing::{trace, warn};

pub fn repeat<T>(gen_tx: GenSender<T>, repeat_value: T)
where
    T: Send + Copy + 'static + Debug,
{
    thread::spawn(move || {
        let mut requested = 0usize;
        loop {
            match gen_tx.req_from_consumer() {
                Ok(GenReq::YieldN(new_requested)) => {
                    requested = new_requested;
                }
                Ok(GenReq::Stop) => {
                    break;
                }
                Ok(GenReq::Yield) => {
                    requested = 1;
                }
                Err(e) => {
                    warn!("Receiver hung up: {}", e);
                }
            }
            for _ in 0..requested {
                match gen_tx.send(repeat_value) {
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
