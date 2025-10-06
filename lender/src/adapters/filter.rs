use core::fmt;

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, FallibleLend, FallibleLender, FallibleLending, FusedLender, Lend, Lender,
    Lending,
};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Filter<L, P> {
    pub(crate) lender: L,
    predicate: P,
}
impl<L, P> Filter<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> Filter<L, P> { Filter { lender, predicate } }
    pub fn into_inner(self) -> L { self.lender }
    pub fn into_parts(self) -> (L, P) { (self.lender, self.predicate) }
}
impl<I: fmt::Debug, P> fmt::Debug for Filter<I, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Filter").field("lender", &self.lender).finish()
    }
}
impl<'lend, L, P> Lending<'lend> for Filter<L, P>
where
    P: FnMut(&Lend<'lend, L>) -> bool,
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}
impl<L, P> Lender for Filter<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> { self.lender.find(&mut self.predicate) }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        #[inline]
        fn f<L: for<'all> Lending<'all>, F: FnMut(&Lend<'_, L>) -> bool>(mut f: F) -> impl FnMut(Lend<'_, L>) -> usize {
            move |x| (f)(&x) as usize
        }
        self.lender.map(f::<Self, _>(self.predicate)).iter().sum()
    }
}
impl<L, P> DoubleEndedLender for Filter<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> { self.lender.rfind(&mut self.predicate) }
}
impl<L, P> FusedLender for Filter<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: FusedLender,
{
}

impl<'lend, L, P> FallibleLending<'lend> for Filter<L, P>
where
    P: FnMut(&FallibleLend<'lend, L>) -> Result<bool, L::Error>,
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}
impl<L, P> FallibleLender for Filter<L, P>
where
    P: FnMut(&FallibleLend<'_, L>) -> Result<bool, L::Error>,
    L: FallibleLender,
{
    type Error = L::Error;

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> { self.lender.find(&mut self.predicate) }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        #[inline]
        fn f<E, L: for<'all> FallibleLending<'all>, F: FnMut(&FallibleLend<'_, L>) -> Result<bool, E>>(
            mut f: F,
        ) -> impl FnMut(FallibleLend<'_, L>) -> Result<usize, E> {
            move |x| (f)(&x).map(|res| res as usize)
        }
        let lender = self.lender.map(f::<_, Self, _>(self.predicate));
        crate::fallible_adapters::NonFallibleAdapter::process(lender, core::iter::Iterator::sum).map_err(|(_, err)| err)
    }
}
impl<L, P> DoubleEndedFallibleLender for Filter<L, P>
where
    P: FnMut(&FallibleLend<'_, L>) -> Result<bool, L::Error>,
    L: DoubleEndedFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> { self.lender.rfind(&mut self.predicate) }
}
