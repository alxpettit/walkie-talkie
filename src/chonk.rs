// TODO: figure out an ADT for Chonk that will let us store
// Need to store both sides of

use itertools::Itertools;

// Remember: the perfect is the enemy of the good
#[derive(Debug)]
struct Chonk<T> {
    data: Vec<T>,
    max_size: usize,
}

impl<T> PartialEq<Self> for Chonk<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> PartialEq<Vec<T>> for Chonk<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Vec<T>) -> bool {
        self.data == *other
    }
}

impl<T> PartialEq<Option<Vec<T>>> for Chonk<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Option<Vec<T>>) -> bool {
        match other {
            None => false,
            Some(data) => self.data == *data,
        }
    }
}

impl<T> Chonk<T> {
    /// Get a new self. Takes a usize for constraining the maximum size of the chonk.
    fn new(max_size: usize) -> Self {
        let data = Vec::with_capacity(max_size);
        Self { data, max_size }
    }

    /// Like set_max_size, but immediately curtails size to enforce the new maximum siz.
    fn do_max_size(&mut self, max_size: usize) -> Option<Vec<T>> {
        self.max_size = max_size;
        self.curtail()
    }

    /// Sets maximum size
    fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
    }

    /// To slurp a Vec, is to consume the elements inside them and append them to yourself
    /// but if you consume too much (exceeding `max_size`), you may find yourself secreting the excess
    /// If it fits nicely, you return `None`, otherwise you return `Some<Vec<T>>`.
    fn slurp(&mut self, other: &mut Vec<T>) -> Option<Vec<T>> {
        self.data.append(other);
        self.curtail()
    }

    /// To ploop a Vec, is to consume the elements inside it and prepend it to yourself
    /// but if you consume too much (exceeding max_size), you may find yourself secreting the excess
    /// If it fits nicely, you return None, otherwise you return Some<Vec<SomeCrap>>.
    /// Unlike slurp, ploop is a more radical and daring option, and a sign of a true fearless warrior.
    /// It is an act of radical self-mastery, a recreation of one's self with new foundations.
    /// It can only be done by one who has achieved true self ownership, as a mere reference is insufficient.
    fn ploop(mut self, other: Vec<T>) -> (Self, Option<Vec<T>>) {
        let mut old_data = self.data;
        self.data = other;
        let curtailed = self.slurp(&mut old_data);

        (self, curtailed)
    }

    /// Splits the end off according to the maximum size of the chonk
    fn curtail(&mut self) -> Option<Vec<T>> {
        if self.data.len() > self.max_size {
            Some(self.data.split_off(self.max_size))
        } else {
            None
        }
    }

    /// Eat a vector, returning self with the internal data being made from that vector,
    /// and with `max_size` set according to the second argument.
    fn from_with_max_size(v: Vec<T>, max_size: usize) -> Self {
        Self { data: v, max_size }
    }
}

impl<T> IntoIterator for Chonk<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<T> From<Vec<T>> for Chonk<T> {
    fn from(value: Vec<T>) -> Self {
        let value_len = value.len();
        Self {
            data: value,
            max_size: value_len,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_chonk() {
        let mut chonk = Chonk::<i32>::new(6);
        let slurp_excess = chonk.slurp(&mut vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        dbg!(&slurp_excess);
        dbg!(&chonk);
        chonk.set_max_size(10);
        let (chonk, ploop_excess) = chonk.ploop(vec![0, 10, 20, 30, 40, 50]);
        dbg!(&chonk);
        dbg!(&ploop_excess);

        assert_eq!(chonk, Some(vec![0, 10, 20, 30, 40, 50, 0, 1, 2, 3]));
        assert_eq!(ploop_excess, Some(vec![4, 5]));
    }
}
