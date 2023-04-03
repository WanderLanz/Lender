use crate::{hkts::HKAFnMutOpt, Lender, Lending};
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct FilterMap<L, F> {
    lender: L,
    f: F,
}
impl<L, F> FilterMap<L, F> {
    pub(crate) fn new(lender: L, f: F) -> FilterMap<L, F> { FilterMap { lender, f } }
}
impl<'lend, B, L, F> Lending<'lend> for FilterMap<L, F>
where
    F: FnMut(<L as Lending<'lend>>::Lend) -> Option<B>,
    B: 'lend,
    L: Lender,
{
    type Lend = B;
}
impl<L, F> Lender for FilterMap<L, F>
where
    for<'all> F: HKAFnMutOpt<'all, <L as Lending<'all>>::Lend>,
    L: Lender,
{
    fn next<'next>(&'next mut self) -> Option<<Self as Lending<'next>>::Lend> {
        while let Some(x) = self.lender.next() {
            if let Some(y) = (self.f)(x) {
                // REVIEW: SAFETY: prevents &mut Self after return
                return Some(unsafe {
                    core::mem::transmute::<<Self as Lending<'_>>::Lend, <Self as Lending<'next>>::Lend>(y)
                });
            }
        }
        None
    }
}
