use core::fmt;

use crate::{higher_order::FnMutHKAOpt, Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Scan<L, St, F> {
    lender: L,
    f: F,
    state: St,
}
impl<L, St, F> Scan<L, St, F> {
    pub(crate) fn new(lender: L, state: St, f: F) -> Scan<L, St, F> { Scan { lender, state, f } }
}
impl<L: fmt::Debug, St: fmt::Debug, F> fmt::Debug for Scan<L, St, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scan").field("lender", &self.lender).field("state", &self.state).finish()
    }
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
    for<'all> F: FnMutHKAOpt<'all, (&'all mut St, <L as Lending<'all>>::Lend)>,
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { (self.f)((&mut self.state, self.lender.next()?)) }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
