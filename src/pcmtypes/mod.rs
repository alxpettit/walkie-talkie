use std::error::Error;

pub type PCMUnit = f32;
pub type PCMResult = Result<PCMUnit, Box<dyn Error>>;
pub type PCMVec = Vec<PCMUnit>;
pub type PCMChunk = [PCMUnit; 100];
