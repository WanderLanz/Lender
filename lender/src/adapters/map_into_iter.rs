use core::{iter::FusedIterator, marker::PhantomData};

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender};

/// An iterator that maps each element of the underlying lender into an owned
/// value.
///
/// This `struct` is created by the
/// [`map_into_iter()`](crate::Lender::map_into_iter) method on [`Lender`].
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MapIntoIter<L, O, F> {
    pub(crate) lender: L,
    pub(crate) f: F,
    pub(crate) _marker: PhantomData<fn() -> O>,
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

    /// Returns the inner lender.
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

impl<L: core::fmt::Debug, O, F> core::fmt::Debug for MapIntoIter<L, O, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MapIntoIter")
            .field("lender", &self.lender)
            .finish_non_exhaustive()
    }
}

impl<L: Lender, O, F: FnMut(Lend<'_, L>) -> O> Iterator for MapIntoIter<L, O, F> {
    type Item = O;
    #[inline(always)]
    fn next(&mut self) -> Option<O> {
        self.lender.next().map(&mut self.f)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<O> {
        self.lender.nth(n).map(&mut self.f)
    }

    #[inline]
    fn fold<B, G>(self, init: B, mut g: G) -> B
    where
        G: FnMut(B, Self::Item) -> B,
    {
        let mut f = self.f;
        self.lender.fold(init, |acc, x| g(acc, f(x)))
    }
}

impl<L: DoubleEndedLender, O, F: FnMut(Lend<'_, L>) -> O> DoubleEndedIterator
    for MapIntoIter<L, O, F>
{
    #[inline(always)]
    fn next_back(&mut self) -> Option<O> {
        self.lender.next_back().map(&mut self.f)
    }

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Option<O> {
        self.lender.nth_back(n).map(&mut self.f)
    }

    #[inline]
    fn rfold<B, G>(self, init: B, mut g: G) -> B
    where
        G: FnMut(B, Self::Item) -> B,
    {
        let mut f = self.f;
        self.lender.rfold(init, |acc, x| g(acc, f(x)))
    }
}

impl<L: ExactSizeLender, O, F: FnMut(Lend<'_, L>) -> O> ExactSizeIterator for MapIntoIter<L, O, F> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }
}

impl<L: FusedLender, O, F: FnMut(Lend<'_, L>) -> O> FusedIterator for MapIntoIter<L, O, F> {}
