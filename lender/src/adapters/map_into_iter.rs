use core::{iter::FusedIterator, marker::PhantomData};

use crate::{
    DoubleEndedLender, ExactSizeLender,
    FusedLender, Lend, Lender,
};

#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct MapIntoIter<L, O, F> {
    pub(crate) lender: L,
    pub(crate) f: F,
    pub(crate) _marker: PhantomData<O>,
}

impl<L, O, F> MapIntoIter<L, O, F> {
    #[inline(always)]
    pub(crate) fn new(lender: L, f: F) -> MapIntoIter<L, O, F> {
        MapIntoIter {
            lender,
            f,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the mapping function.
    #[inline(always)]
    pub fn into_parts(self) -> (L, F) {
        (self.lender, self.f)
    }
}

impl<L: Lender, O, F: FnMut(Lend<'_, L>) -> O> Iterator for MapIntoIter<L, O, F> {
    type Item = O;
    #[inline]
    fn next(&mut self) -> Option<O> {
        self.lender.next().map(&mut self.f)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}

impl<L: DoubleEndedLender, O, F: FnMut(Lend<'_, L>) -> O> DoubleEndedIterator
    for MapIntoIter<L, O, F>
{
    #[inline]
    fn next_back(&mut self) -> Option<O> {
        self.lender.next_back().map(&mut self.f)
    }
}

impl<L: ExactSizeLender, O, F: FnMut(Lend<'_, L>) -> O> ExactSizeIterator for MapIntoIter<L, O, F> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }
}

impl<L: FusedLender, O, F: FnMut(Lend<'_, L>) -> O> FusedIterator for MapIntoIter<L, O, F> {}

