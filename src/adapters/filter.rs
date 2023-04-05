use core::fmt;

use crate::{DoubleEndedLender, FusedLender, Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Filter<L, P> {
    pub(crate) lender: L,
    predicate: P,
}
impl<L, P> Filter<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> Filter<L, P> { Filter { lender, predicate } }
}
impl<I: fmt::Debug, P> fmt::Debug for Filter<I, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Filter").field("lender", &self.lender).finish()
    }
}
impl<'lend, L, P> Lending<'lend> for Filter<L, P>
where
    P: FnMut(&<L as Lending<'lend>>::Lend) -> bool,
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L, P> Lender for Filter<L, P>
where
    P: FnMut(&<L as Lending<'_>>::Lend) -> bool,
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.lender.find(&mut self.predicate) }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
    #[inline]
    fn count(mut self) -> usize
    where
        Self: Sized,
    {
        let p = &mut self.predicate;
        self.lender.map(move |x| (p)(&x) as usize).iter().sum()
    }
}
impl<L, P> DoubleEndedLender for Filter<L, P>
where
    P: FnMut(&<L as Lending<'_>>::Lend) -> bool,
    L: DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.lender.rfind(&mut self.predicate) }
}
impl<L, P> FusedLender for Filter<L, P>
where
    P: FnMut(&<L as Lending<'_>>::Lend) -> bool,
    L: FusedLender,
{
}
