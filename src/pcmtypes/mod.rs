use next_gen::generator::Generator;
use std::error::Error;
use std::pin::Pin;

pub type PCMUnit = f32;
pub type PCMResult = Result<PCMUnit, Box<dyn Error>>;
pub type PCMVec = Vec<PCMUnit>;
pub type PCMGenerator<'g> = Pin<&'g mut dyn Generator<Yield = PCMUnit, Return = ()>>;
