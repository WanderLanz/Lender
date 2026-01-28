use core::iter::FusedIterator;

use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeLender, FallibleLender, FallibleLending,
    FusedLender, Lender, Lending,
};

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Cloned<L> {
    lender: L,
}
impl<L> Cloned<L> {
    pub(crate) fn new(lender: L) -> Cloned<L> {
        Cloned { lender }
    }

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
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.lender.next().cloned()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}

impl<T, L> DoubleEndedIterator for Cloned<L>
where
    L: DoubleEndedLender,
    T: Clone,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.lender.next_back().cloned()
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
    fn default() -> Self {
        Self::new(L::default())
    }
}

impl<T, L> FallibleIterator for Cloned<L>
where
    L: FallibleLender,
    T: Clone,
    L: for<'all> FallibleLending<'all, Lend = &'all T>,
{
    type Item = T;
    type Error = L::Error;

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.lender.next().map(Option::<&T>::cloned)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}

impl<T, L> DoubleEndedFallibleIterator for Cloned<L>
where
    L: DoubleEndedFallibleLender,
    T: Clone,
    L: for<'all> FallibleLending<'all, Lend = &'all T>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.lender.next_back().map(Option::<&T>::cloned)
    }
}
