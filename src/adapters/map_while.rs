use crate::{hkts::HKAFnMutOpt, Lender, Lending};
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MapWhile<L, P> {
    lender: L,
    predicate: P,
}
impl<L, P> MapWhile<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> MapWhile<L, P> { MapWhile { lender, predicate } }
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
    P: for<'all> HKAFnMutOpt<'all, <L as Lending<'all>>::Lend>,
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { (self.predicate)(self.lender.next()?) }
}
