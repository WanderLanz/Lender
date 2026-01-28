use core::iter::FusedIterator;

use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeLender, FallibleLender, FallibleLending,
    FusedLender, Lender, Lending,
};

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Copied<L> {
    lender: L,
}
impl<L> Copied<L> {
    pub(crate) fn new(lender: L) -> Copied<L> {
        Copied { lender }
    }

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
    fn default() -> Self {
        Self::new(L::default())
    }
}

impl<T, L> FallibleIterator for Copied<L>
where
    L: FallibleLender,
    T: Copy,
    L: for<'all> FallibleLending<'all, Lend = &'all T>,
{
    type Item = T;
    type Error = L::Error;

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.lender.next().map(Option::<&T>::copied)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}
impl<T, L> DoubleEndedFallibleIterator for Copied<L>
where
    L: DoubleEndedFallibleLender,
    T: Copy,
    L: for<'all> FallibleLending<'all, Lend = &'all T>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.lender.next_back().map(Option::<&T>::copied)
    }
}
