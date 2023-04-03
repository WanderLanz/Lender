use crate::{Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct TakeWhile<L, P> {
    lender: L,
    flag: bool,
    predicate: P,
}
impl<L, P> TakeWhile<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> TakeWhile<L, P> { TakeWhile { lender, flag: false, predicate } }
}
impl<'lend, L, P> Lending<'lend> for TakeWhile<L, P>
where
    P: FnMut(&<L as Lending<'lend>>::Lend) -> bool,
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
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
}
