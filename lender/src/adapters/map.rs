use core::fmt;

use crate::{higher_order::FnMutHKA, DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Map<L, F> {
    pub(crate) lender: L,
    f: F,
}
impl<L, F> Map<L, F> {
    pub(crate) fn new(lender: L, f: F) -> Map<L, F> {
        Map { lender, f }
    }
    pub fn into_inner(self) -> L {
        self.lender
    }
    pub fn into_parts(self) -> (L, F) {
        (self.lender, self.f)
    }
}
impl<L: fmt::Debug, F> fmt::Debug for Map<L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map").field("lender", &self.lender).finish()
    }
}
impl<'lend, L, F> Lending<'lend> for Map<L, F>
where
    F: for<'all> FnMutHKA<'all, Lend<'all, L>>,
    L: Lender,
{
    type Lend = <F as FnMutHKA<'lend, Lend<'lend, L>>>::B;
}
impl<L, F> Lender for Map<L, F>
where
    F: for<'all> FnMutHKA<'all, Lend<'all, L>>,
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next().map(&mut self.f)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}
impl<L: DoubleEndedLender, F> DoubleEndedLender for Map<L, F>
where
    F: for<'all> FnMutHKA<'all, Lend<'all, L>>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next_back().map(&mut self.f)
    }
}
impl<L: ExactSizeLender, F> ExactSizeLender for Map<L, F>
where
    F: for<'all> FnMutHKA<'all, Lend<'all, L>>,
{
    #[inline]
    fn len(&self) -> usize {
        self.lender.len()
    }
    #[inline]
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
    }
}
impl<L: FusedLender, F> FusedLender for Map<L, F> where F: for<'all> FnMutHKA<'all, Lend<'all, L>> {}
// impl<I, L, F> IntoIterator for Map<L, F>
// where
//     L: Lender,
//     F: for<'all> HKAFnMut<'all, Lend<'all, L>, B = I>,
//     I: 'static,
// {
//     type Item = I;
//     type IntoIter = Iter<Self>;
//     #[inline]
//     fn into_iter(self) -> Iter<Self> { Iter::new(self) }
// }
