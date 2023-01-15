use std::error::Error;

pub type Frame = [f32; 8];
pub type PCMUnit = f32;
pub type UnknownFrame = Box<[f32]>;
pub type PCMResult = Result<PCMUnit, Box<dyn Error>>;
pub type PCMVec = Vec<PCMUnit>;
