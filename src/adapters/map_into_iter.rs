use core::iter::FusedIterator;

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender};

#[derive(Clone, Debug)]
pub struct MapIntoIter<L, O, F>
where
    L: Lender,
    F: FnMut(Lend<'_, L>) -> O,
{
    pub(crate) lender: L,
    f: F,
}

impl<L: Lender, O, F: FnMut(Lend<'_, L>) -> O> MapIntoIter<L, O, F> {
    pub(crate) fn new(lender: L, f: F) -> MapIntoIter<L, O, F> {
        MapIntoIter { lender, f }
    }
}

impl<L: Lender, O, F: FnMut(Lend<'_, L>) -> O> Iterator for MapIntoIter<L, O, F> {
    type Item = O;
    #[inline]
    fn next(&mut self) -> Option<O> {
        self.lender.next().map(&mut self.f)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}

impl<L: DoubleEndedLender, O, F: FnMut(Lend<'_, L>) -> O> DoubleEndedIterator for MapIntoIter<L, O, F> {
    #[inline]
    fn next_back(&mut self) -> Option<O> {
        self.lender.next_back().map(&mut self.f)
    }
}

impl<L: ExactSizeLender, O, F: FnMut(Lend<'_, L>) -> O> ExactSizeIterator for MapIntoIter<L, O, F> {
    #[inline]
    fn len(&self) -> usize {
        self.lender.len()
    }
}

impl<L: FusedLender, O, F: FnMut(Lend<'_, L>) -> O> FusedIterator for MapIntoIter<L, O, F> {}
