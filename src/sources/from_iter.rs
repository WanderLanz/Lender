use core::iter::FusedIterator;

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};

/// Creates a lender from an iterator.
#[inline]
pub fn from_iter<I: Iterator>(iter: I) -> FromIter<I> { FromIter { iter } }

/// A lender that yields elements from an iterator.
///
/// This `struct` is created by the [`from_iter()`] function.
/// See its documentation for more.

#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FromIter<I> {
    iter: I,
}

impl<'lend, I: Iterator> Lending<'lend> for FromIter<I> {
    type Lend = I::Item;
}

impl<I: Iterator> Lender for FromIter<I> {
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.iter.next() }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<I: DoubleEndedIterator> DoubleEndedLender for FromIter<I> {
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.iter.next_back() }
}

impl<I: ExactSizeIterator> ExactSizeLender for FromIter<I> {
    fn len(&self) -> usize { self.iter.len() }
}

impl<I: FusedIterator> FusedLender for FromIter<I> {}
