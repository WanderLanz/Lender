use crate::{DoubleEndedLender, Lender, Lending};
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Rev<L> {
    lender: L,
}
impl<L> Rev<L> {
    pub(crate) fn new(lender: L) -> Rev<L> { Rev { lender } }
}
impl<'lend, L> Lending<'lend> for Rev<L>
where
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L> Lender for Rev<L>
where
    L: DoubleEndedLender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.lender.next_back() }
}
