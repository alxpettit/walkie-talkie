use std::error::Error;
use std::slice::Iter;

pub type PCMUnit = f32;
pub type PCMResult = Result<PCMUnit, Box<dyn Error>>;
//pub type PCMVec = Vec<PCMUnit>;

pub struct PCMVec {
    data: Vec<PCMUnit>,
}

impl PCMVec {
    pub fn borrow_mut_inner(&mut self) -> &mut Vec<PCMUnit> {
        &mut self.data
    }
    pub fn borrow_inner(&self) -> &Vec<PCMUnit> {
        &self.data
    }
    pub fn iter(&self) -> Iter<PCMUnit> {
        self.data.iter()
    }
}

impl Into<Vec<PCMUnit>> for PCMVec {
    fn into(self) -> Vec<PCMUnit> {
        self.data
    }
}

impl Into<PCMVec> for Vec<PCMUnit> {
    fn into(self) -> PCMVec {
        PCMVec { data: self }
    }
}
