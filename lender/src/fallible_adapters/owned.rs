use alloc::borrow::ToOwned;

use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

use crate::{DoubleEndedFallibleLender, FallibleLend, FallibleLender, Owned};

impl<T, L> FallibleIterator for Owned<L>
where
    L: FallibleLender,
    for<'all> FallibleLend<'all, L>: ToOwned<Owned = T>,
{
    type Item = T;
    type Error = L::Error;

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        Ok(self.lender.next()?.map(|ref x| x.to_owned()))
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
        self.lender.fold(init, |acc, ref x| f(acc, x.to_owned()))
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.lender.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<Self::Item>, Self::Error> {
        Ok(self.lender.nth(n)?.map(|ref x| x.to_owned()))
    }
}

impl<T, L> DoubleEndedFallibleIterator for Owned<L>
where
    L: DoubleEndedFallibleLender,
    for<'all> FallibleLend<'all, L>: ToOwned<Owned = T>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        Ok(self.lender.next_back()?.map(|ref x| x.to_owned()))
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> Result<B, Self::Error>,
    {
        self.lender.rfold(init, |acc, ref x| f(acc, x.to_owned()))
    }
}
