use core::iter::FusedIterator;

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};

/// An iterator that copies the elements of an underlying lender.
///
/// This `struct` is created by the [`copied()`](crate::Lender::copied) method
/// on [`Lender`].
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Copied<L> {
    pub(crate) lender: L,
}

impl<L> Copied<L> {
    #[inline(always)]
    pub(crate) fn new(lender: L) -> Copied<L> {
        Copied { lender }
    }

    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }
}

impl<T, L> Iterator for Copied<L>
where
    L: Lender,
    T: Copy,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.lender.next().copied()
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
        self.lender.fold(init, |acc, x| f(acc, *x))
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.lender.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.lender.nth(n).copied()
    }
}

impl<T, L> DoubleEndedIterator for Copied<L>
where
    L: DoubleEndedLender,
    T: Copy,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.lender.next_back().copied()
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.lender.rfold(init, |acc, x| f(acc, *x))
    }
}

impl<T, L> ExactSizeIterator for Copied<L>
where
    L: ExactSizeLender,
    T: Copy,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }
}

impl<T, L> FusedIterator for Copied<L>
where
    L: FusedLender,
    T: Copy,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
}

impl<L> Default for Copied<L>
where
    L: Default,
{
    #[inline(always)]
    fn default() -> Self {
        Self::new(L::default())
    }
}
