use crate::{Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct SkipWhile<L, P> {
    lender: L,
    flag: bool,
    predicate: P,
}
impl<L, P> SkipWhile<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> SkipWhile<L, P> { SkipWhile { lender, flag: false, predicate } }
}
impl<'lend, L, P> Lending<'lend> for SkipWhile<L, P>
where
    P: FnMut(&<L as Lending<'lend>>::Lend) -> bool,
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L, P> Lender for SkipWhile<L, P>
where
    P: FnMut(&<L as Lending<'_>>::Lend) -> bool,
    L: Lender,
{
    #[inline]
    fn next<'next>(&'next mut self) -> Option<<Self as Lending<'next>>::Lend> {
        if self.flag {
            self.lender.next()
        } else {
            while let Some(x) = self.lender.next() {
                if !(self.predicate)(&x) {
                    self.flag = true;
                    // SAFETY: only lives until the next call to next
                    return Some(unsafe {
                        core::mem::transmute::<<Self as Lending<'_>>::Lend, <Self as Lending<'next>>::Lend>(x)
                    });
                }
            }
            None
        }
    }
}
