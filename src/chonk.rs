// TODO: figure out an ADT for Chonk that will let us store
// Need to store both sides of

use crate::StreamExt;
use futures_core::Stream;
use itertools::{repeat_n, Itertools};
use std::collections::VecDeque;

// Remember: the perfect is the enemy of the good
#[derive(Debug)]
pub struct Chonk<T> {
    data: VecDeque<T>,
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

impl<T> PartialEq<VecDeque<T>> for Chonk<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &VecDeque<T>) -> bool {
        self.data == *other
    }
}

impl<T> PartialEq<ChonkRemainder<T>> for Chonk<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &ChonkRemainder<T>) -> bool {
        self.data == other.data.unwrap_or_default()
    }
}

// }
//
// impl<T> PartialEq<ChonkRemainder<T>> for Chonk<T>
// where
//     T: PartialEq,
// {
//     fn eq(&self, other: &ChonkRemainder<T>) -> bool {
//         Some(&self.data) == other.data.as_ref()
//     }
// }

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ChonkRemainder<T> {
    data: Option<VecDeque<T>>,
}

impl<T> ChonkRemainder<T> {
    pub(crate) fn new() -> Self {
        Self { data: None }
    }

    fn has_none(&self) -> bool {
        return self.data.is_none();
    }

    fn has_some(&self) -> bool {
        return self.data.is_some();
    }

    fn extract_vecdeque(mut self) -> VecDeque<T> {
        match self.data {
            Some(data) => data,
            None => VecDeque::<T>::new(),
        }
    }
}

impl<T> From<Option<VecDeque<T>>> for ChonkRemainder<T> {
    fn from(value: Option<VecDeque<T>>) -> Self {
        Self { data: value }
    }
}

impl<T> From<VecDeque<T>> for ChonkRemainder<T> {
    fn from(value: VecDeque<T>) -> Self {
        Self { data: Some(value) }
    }
}

impl<T> From<ChonkRemainder<T>> for VecDeque<T> {
    fn from(value: ChonkRemainder<T>) -> Self {
        match value.data {
            Some(data) => data,
            None => VecDeque::<T>::new(),
        }
    }
}

impl<T> Chonk<T> {
    /// Get a new self. Takes a usize for constraining the maximum size of the chonk.
    pub fn new(max_size: usize) -> Self {
        let data = VecDeque::with_capacity(max_size);
        Self { data, max_size }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Like set_max_size, but immediately curtails size to enforce the new maximum siz.
    pub fn do_max_size(&mut self, max_size: usize) -> ChonkRemainder<T> {
        self.max_size = max_size;
        self.curtail()
    }

    /// Sets maximum size
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
    }

    pub fn new_repeat_n(element: T, n: usize) -> Self
    where
        T: Clone,
    {
        Self {
            data: repeat_n(element, n).collect::<VecDeque<_>>(),
            max_size: n,
        }
    }

    /// To slurp a Vec, is to consume the elements inside them and append them to yourself
    /// but if you consume too much (exceeding `max_size`), you may find yourself secreting the excess
    /// If it fits nicely, you return `None`, otherwise you return `Some<Vec<T>>`.
    pub fn slurp(&mut self, other: &mut VecDeque<T>) -> ChonkRemainder<T> {
        self.data.append(other);
        self.curtail()
    }

    /// To `ploop` a `ChonkRemainder`, is to consume the elements inside it and prepend it to yourself
    /// but if you consume too much (exceeding `max_size`), you may find yourself secreting
    /// the excess into your return `ChonkRemainder`.
    /// Unlike `slurp`, `ploop` is a more radical and daring option, and a sign of a true fearless warrior.
    /// It is an act of radical self-mastery, a recreation of one's self with new foundations.
    /// It can only be done by one who has achieved true self ownership, as a mere reference is insufficient.
    pub fn ploop(mut self, other: ChonkRemainder<T>) -> (Self, ChonkRemainder<T>) {
        let mut old_data = self.data;
        self.data = other.data.unwrap_or_default();
        let curtailed = self.slurp(&mut old_data);

        (self, curtailed)
    }

    pub fn pop_front_into(mut self, arr: &mut [T]) -> Result<(), ()> {
        for p in arr {
            self.data.pop_back();
        }
        todo!()
    }

    /// Splits the end off according to the maximum size of the chonk
    pub fn curtail(&mut self) -> ChonkRemainder<T> {
        if self.data.len() > self.max_size {
            ChonkRemainder::<T>::from(self.data.split_off(self.max_size))
        } else {
            ChonkRemainder::<T>::from(None)
        }
    }

    /// Eat a vector, returning self with the internal data being made from that vector,
    /// and with `max_size` set according to the second argument.
    pub fn from_with_max_size(v: VecDeque<T>, max_size: usize) -> Self {
        Self { data: v, max_size }
    }

    pub async fn nom_stream_ref<S: Stream<Item = T> + Unpin>(&mut self, input: &mut S) {
        let take = self.data.len()..self.max_size;
        'buf: for _ in take {
            match input.next().await {
                Some(x) => self.data.push_back(x),
                None => {
                    break 'buf;
                }
            }
        }
    }

    /// Unlike `slurp`, and `ploop`, `nom` is a precision operation. `nom` never takes more than needed.
    /// I _think_ this should work on async Stream too, but I'm not sure.
    /// If not, I'll make a method for that.
    pub fn nom_iter<I: Iterator<Item = T>>(&mut self, input: I) {
        let mut taken = input
            .take(self.max_size - self.data.len())
            .collect::<VecDeque<_>>();
        self.data.append(&mut taken);
    }

    /// Run function against every value we have
    pub fn for_each_mut<F: Fn(&mut T)>(&mut self, f: F) {
        for x in self.data.iter_mut() {
            f(x);
        }
    }
}

impl<T> IntoIterator for Chonk<T> {
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<T> From<VecDeque<T>> for Chonk<T> {
    fn from(value: VecDeque<T>) -> Self {
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
        let slurp_excess = chonk.slurp(&mut VecDeque::from(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
        ]));
        dbg!(&slurp_excess);
        dbg!(&chonk);
        chonk.set_max_size(10);
        let (mut chonk, ploop_excess) = chonk.ploop(ChonkRemainder::from(VecDeque::from(vec![
            0, 10, 20, 30, 40, 50,
        ])));
        dbg!(&chonk);
        dbg!(&ploop_excess);

        assert_eq!(
            chonk,
            ChonkRemainder::from(VecDeque::from(vec![0, 10, 20, 30, 40, 50, 0, 1, 2, 3]))
        );
        assert_eq!(
            ploop_excess,
            ChonkRemainder::from(VecDeque::from(vec![4, 5]))
        );

        chonk.set_max_size(32);
        let iter = std::iter::repeat(100);
        chonk.nom_iter(iter);
        dbg!(&chonk);
        assert_eq!(chonk.len(), 32);

        for x in chonk.into_iter() {
            dbg!(x);
        }
    }
}
