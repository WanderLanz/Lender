use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

use crate::{Cloned, DoubleEndedFallibleLender, FallibleLender, FallibleLending};

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

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> Result<B, Self::Error>,
    {
        self.lender.fold(init, |acc, x| f(acc, x.clone()))
    }

    #[inline(always)]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.lender.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<Self::Item>, Self::Error> {
        self.lender.nth(n).map(Option::<&T>::cloned)
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

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> Result<B, Self::Error>,
    {
        self.lender.rfold(init, |acc, x| f(acc, x.clone()))
    }
}
