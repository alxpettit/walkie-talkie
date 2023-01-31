use std::error::Error;
use std::slice::Iter;

pub type PCMUnit = f32;
pub type PCMResult = Result<PCMUnit, Box<dyn Error>>;
//pub type PCMVec = Vec<PCMUnit>;

pub struct PCMVec {
    data: Vec<PCMUnit>,
}

impl PCMVec {
    pub fn new() -> Self {
        Self {
            data: Vec::<PCMUnit>::new(),
        }
    }
    pub fn append(&mut self, other: &mut Self) {
        self.append(other);
    }
    pub fn borrow_mut_inner(&mut self) -> &mut Vec<PCMUnit> {
        &mut self.data
    }
    pub fn borrow_inner(&self) -> &Vec<PCMUnit> {
        &self.data
    }
    pub fn iter(&self) -> Iter<PCMUnit> {
        self.data.iter()
    }
    pub fn pop(&mut self) -> Option<PCMUnit> {
        self.data.pop()
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
