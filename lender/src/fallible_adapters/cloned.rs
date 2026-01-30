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
