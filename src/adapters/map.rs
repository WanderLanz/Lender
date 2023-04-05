use core::fmt;

use crate::{hkts::HKAFnMut, DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Map<L, F> {
    pub(crate) lender: L,
    f: F,
}
impl<L, F> Map<L, F> {
    pub(crate) fn new(lender: L, f: F) -> Map<L, F> { Map { lender, f } }
}
impl<L: fmt::Debug, F> fmt::Debug for Map<L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("Map").field("lender", &self.lender).finish() }
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
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.lender.next().map(&mut self.f) }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
}
impl<L: DoubleEndedLender, F> DoubleEndedLender for Map<L, F>
where
    F: for<'all> HKAFnMut<'all, <L as Lending<'all>>::Lend>,
{
    #[inline]
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.lender.next_back().map(&mut self.f) }
}
impl<L: ExactSizeLender, F> ExactSizeLender for Map<L, F>
where
    F: for<'all> HKAFnMut<'all, <L as Lending<'all>>::Lend>,
{
    #[inline]
    fn len(&self) -> usize { self.lender.len() }
    #[inline]
    fn is_empty(&self) -> bool { self.lender.is_empty() }
}
impl<L: FusedLender, F> FusedLender for Map<L, F> where F: for<'all> HKAFnMut<'all, <L as Lending<'all>>::Lend> {}
// impl<I, L, F> IntoIterator for Map<L, F>
// where
//     L: Lender,
//     F: for<'all> HKAFnMut<'all, <L as Lending<'all>>::Lend, B = I>,
//     I: 'static,
// {
//     type Item = I;
//     type IntoIter = Iter<Self>;
//     #[inline]
//     fn into_iter(self) -> Iter<Self> { Iter::new(self) }
// }
