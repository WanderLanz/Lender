use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

use crate::{DoubleEndedFallibleLender, FallibleLend, FallibleLender, MapIntoIter};

impl<L, O, F> FallibleIterator for MapIntoIter<L, O, F>
where
    L: FallibleLender,
    F: FnMut(FallibleLend<'_, L>) -> Result<O, L::Error>,
{
    type Item = O;
    type Error = L::Error;
    #[inline]
    fn next(&mut self) -> Result<Option<O>, Self::Error> {
        self.lender.next()?.map(&mut self.f).transpose()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}

impl<L, O, F> DoubleEndedFallibleIterator for MapIntoIter<L, O, F>
where
    L: DoubleEndedFallibleLender,
    F: FnMut(FallibleLend<'_, L>) -> Result<O, L::Error>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<O>, Self::Error> {
        self.lender.next_back()?.map(&mut self.f).transpose()
    }
}
