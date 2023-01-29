// TODO: figure out an ADT for Chonk that will let us store
// Need to store both sides of

use itertools::Itertools;

// Remember: the perfect is the enemy of the good
struct Chonk<T> {
    data: Vec<T>,
    max_size: usize,
}

impl<T> Chonk<T> {
    fn new(max_size: usize) -> Self {
        let data = Vec::with_capacity(max_size);
        Self { data, max_size }
    }
    fn slurp(&mut self, other: &mut Vec<T>) -> Vec<T> {
        self.data.append(other);
        if self.data.len() > self.max_size {
            self.data.split_off(self.max_size)
        } else {
            Vec::new()
        }
    }
}

impl<T> IntoIterator for Chonk<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}
