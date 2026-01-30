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
}
