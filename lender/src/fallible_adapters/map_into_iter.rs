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

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<O>, Self::Error> {
        self.lender.nth(n)?.map(&mut self.f).transpose()
    }

    #[inline]
    fn fold<B, G>(self, init: B, mut g: G) -> Result<B, Self::Error>
    where
        Self: Sized,
        G: FnMut(B, Self::Item) -> Result<B, Self::Error>,
    {
        let mut f = self.f;
        self.lender.fold(init, |acc, x| g(acc, (f)(x)?))
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

    #[inline]
    fn rfold<B, G>(self, init: B, mut g: G) -> Result<B, Self::Error>
    where
        Self: Sized,
        G: FnMut(B, Self::Item) -> Result<B, Self::Error>,
    {
        let mut f = self.f;
        self.lender.rfold(init, |acc, x| g(acc, (f)(x)?))
    }
}
