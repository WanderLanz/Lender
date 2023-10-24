use core::fmt;

use crate::{FusedLender, Lend, Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct TakeWhile<L, P> {
    lender: L,
    flag: bool,
    predicate: P,
}
impl<L, P> TakeWhile<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> TakeWhile<L, P> {
        TakeWhile { lender, flag: false, predicate }
    }
}
impl<L: fmt::Debug, P> fmt::Debug for TakeWhile<L, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TakeWhile").field("lender", &self.lender).field("flag", &self.flag).finish()
    }
}
impl<'lend, L, P> Lending<'lend> for TakeWhile<L, P>
where
    P: FnMut(&Lend<'lend, L>) -> bool,
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}
impl<L, P> Lender for TakeWhile<L, P>
where
    P: FnMut(&<L as Lending<'_>>::Lend) -> bool,
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if !self.flag {
            let x = self.lender.next()?;
            if (self.predicate)(&x) {
                return Some(x);
            }
            self.flag = true;
        }
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.flag {
            (0, Some(0))
        } else {
            let (_, upper) = self.lender.size_hint();
            (0, upper)
        }
    }
}
impl<L, P> FusedLender for TakeWhile<L, P>
where
    P: FnMut(&<L as Lending<'_>>::Lend) -> bool,
    L: Lender,
{
}
