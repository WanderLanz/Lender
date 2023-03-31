use crate::{hkts::HKFnMutOpt, Lender, Lending};
pub struct Scan<L, St, F> {
    lender: L,
    f: F,
    state: St,
}
impl<L, St, F> Scan<L, St, F> {
    pub(crate) fn new(lender: L, state: St, f: F) -> Scan<L, St, F> { Scan { lender, state, f } }
}
impl<'lend, B, L, St, F> Lending<'lend> for Scan<L, St, F>
where
    F: FnMut((&'lend mut St, <L as Lending<'lend>>::Lend)) -> Option<B>,
    L: Lender,
    B: 'lend,
{
    type Lend = B;
}
impl<L, St, F> Lender for Scan<L, St, F>
where
    for<'all> F: HKFnMutOpt<'all, (&'all mut St, <L as Lending<'all>>::Lend)>,
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { (self.f)((&mut self.state, self.lender.next()?)) }
}
