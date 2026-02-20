use core::iter::FusedIterator;

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};

/// An iterator that clones the elements of an underlying lender.
///
/// This `struct` is created by the [`cloned()`](crate::Lender::cloned) method
/// on [`Lender`].
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Cloned<L> {
    pub(crate) lender: L,
}

impl<L> Cloned<L> {
    #[inline(always)]
    pub(crate) fn new(lender: L) -> Cloned<L> {
        Cloned { lender }
    }

    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }
}

impl<T, L> Iterator for Cloned<L>
where
    L: Lender,
    T: Clone,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
    type Item = T;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.lender.next().cloned()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.lender.fold(init, |acc, x| f(acc, x.clone()))
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.lender.count()
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.lender.nth(n).cloned()
    }
}

impl<T, L> DoubleEndedIterator for Cloned<L>
where
    L: DoubleEndedLender,
    T: Clone,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.lender.next_back().cloned()
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.lender.rfold(init, |acc, x| f(acc, x.clone()))
    }
}

impl<T, L> ExactSizeIterator for Cloned<L>
where
    L: ExactSizeLender,
    T: Clone,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }
}

impl<T, L> FusedIterator for Cloned<L>
where
    L: FusedLender,
    T: Clone,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
}

impl<L> Default for Cloned<L>
where
    L: Default,
{
    #[inline(always)]
    fn default() -> Self {
        Self::new(L::default())
    }
}
