use core::fmt;

use crate::{higher_order::FnMutHKAOpt, Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct MapWhile<L, P> {
    lender: L,
    predicate: P,
}
impl<L, P> MapWhile<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> MapWhile<L, P> { MapWhile { lender, predicate } }
}
impl<L: fmt::Debug, P> fmt::Debug for MapWhile<L, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapWhile").field("lender", &self.lender).finish()
    }
}
impl<'lend, B, L, P> Lending<'lend> for MapWhile<L, P>
where
    P: FnMut(<L as Lending<'lend>>::Lend) -> Option<B>,
    L: Lender,
    B: 'lend,
{
    type Lend = B;
}
impl<L, P> Lender for MapWhile<L, P>
where
    P: for<'all> FnMutHKAOpt<'all, <L as Lending<'all>>::Lend>,
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { (self.predicate)(self.lender.next()?) }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
