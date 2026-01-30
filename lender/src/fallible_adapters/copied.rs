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
