use crate::{Lender, Lending};
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Skip<L> {
    lender: L,
    n: usize,
}
impl<L> Skip<L> {
    pub(crate) fn new(lender: L, n: usize) -> Skip<L> { Skip { lender, n } }
}
impl<'lend, L> Lending<'lend> for Skip<L>
where
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L> Lender for Skip<L>
where
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if self.n > 0 {
            self.lender.nth(core::mem::take(&mut self.n))
        } else {
            self.lender.next()
        }
    }
}
