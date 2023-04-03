use crate::{hkts::HKAFnMut, Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Map<L, F> {
    pub(crate) lender: L,
    f: F,
}
impl<L, F> Map<L, F> {
    pub(crate) fn new(lender: L, f: F) -> Map<L, F> { Map { lender, f } }
}
impl<'lend, B, L, F> Lending<'lend> for Map<L, F>
where
    F: FnMut(<L as Lending<'lend>>::Lend) -> B,
    L: Lender,
    B: 'lend,
{
    type Lend = B;
}
impl<L, F> Lender for Map<L, F>
where
    F: for<'all> HKAFnMut<'all, <L as Lending<'all>>::Lend>,
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.lender.next().map(|x| (self.f)(x)) }
}
