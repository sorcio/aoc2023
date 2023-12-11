/// Iterate over all unique pairs of elements in a slice
pub(crate) struct PairsIterator<'a, T> {
    slice: &'a [T],
    index1: usize,
    index2: usize,
}

impl<'a, T> PairsIterator<'a, T> {
    fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            index1: 0,
            index2: 1,
        }
    }
}

impl<'a, T> Iterator for PairsIterator<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index1 < self.slice.len() {
            if self.index2 < self.slice.len() {
                let pair = (&self.slice[self.index1], &self.slice[self.index2]);
                self.index2 += 1;
                Some(pair)
            } else {
                self.index1 += 1;
                self.index2 = self.index1 + 1;
                self.next()
            }
        } else {
            None
        }
    }
}

pub(crate) trait SliceUtils<T> {
    fn pairs(&self) -> PairsIterator<T>;
}

impl<T> SliceUtils<T> for [T] {
    fn pairs(&self) -> PairsIterator<T> {
        PairsIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_iterator() {
        let mut iter = PairsIterator::new(&[1, 2, 3, 4]);
        assert_eq!(iter.next(), Some((&1, &2)));
        assert_eq!(iter.next(), Some((&1, &3)));
        assert_eq!(iter.next(), Some((&1, &4)));
        assert_eq!(iter.next(), Some((&2, &3)));
        assert_eq!(iter.next(), Some((&2, &4)));
        assert_eq!(iter.next(), Some((&3, &4)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn pairs_iterator_too_small() {
        let mut iter = PairsIterator::new(&[1]);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn pairs_iterator_empty() {
        let mut iter = PairsIterator::new(&[1]);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
