pub mod denoise;
pub mod fft;
pub mod mic;
pub mod pcmtypes;
pub mod speaker;
pub mod vol_up;

use crate::pcmtypes::PCMUnit;
use async_fn_stream::{fn_stream, try_fn_stream};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BuildStreamError, Device, PlayStreamError, StreamConfig, StreamError};
use futures::Stream;
use futures::StreamExt;
use snafu::prelude::*;
use std::sync::mpsc::{Receiver, SendError};
