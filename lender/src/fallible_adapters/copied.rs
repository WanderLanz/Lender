use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

use crate::{Copied, DoubleEndedFallibleLender, FallibleLender, FallibleLending};

impl<T, L> FallibleIterator for Copied<L>
where
    L: FallibleLender,
    T: Copy,
    L: for<'all> FallibleLending<'all, Lend = &'all T>,
{
    type Item = T;
    type Error = L::Error;

    #[inline(always)]
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.lender.next().map(Option::<&T>::copied)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> Result<B, Self::Error>,
    {
        self.lender.fold(init, |acc, x| f(acc, *x))
    }

    #[inline(always)]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.lender.count()
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Result<Option<Self::Item>, Self::Error> {
        self.lender.nth(n).map(Option::<&T>::copied)
    }
}

impl<T, L> DoubleEndedFallibleIterator for Copied<L>
where
    L: DoubleEndedFallibleLender,
    T: Copy,
    L: for<'all> FallibleLending<'all, Lend = &'all T>,
{
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.lender.next_back().map(Option::<&T>::copied)
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> Result<B, Self::Error>,
    {
        self.lender.rfold(init, |acc, x| f(acc, *x))
    }
}
