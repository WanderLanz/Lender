use crate::{Lender, Lending};
pub struct Filter<L, P> {
    pub(crate) lender: L,
    predicate: P,
}
impl<L, P> Filter<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> Filter<L, P> { Filter { lender, predicate } }
}
impl<'lend, L, P> Lending<'lend> for Filter<L, P>
where
    P: FnMut(&<L as Lending<'lend>>::Lend) -> bool,
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L, P> Lender for Filter<L, P>
where
    P: FnMut(&<L as Lending<'_>>::Lend) -> bool,
    L: Lender,
{
    #[inline]
    fn next<'next>(&'next mut self) -> Option<<Self as Lending<'next>>::Lend> {
        while let Some(x) = self.lender.next() {
            if (self.predicate)(&x) {
                // REVIEW: SAFETY: `x` is a reference to a value in `self.lender`, which is valid for `'next`
                return Some(unsafe {
                    core::mem::transmute::<<Self as Lending<'_>>::Lend, <Self as Lending<'next>>::Lend>(x)
                });
            }
        }
        None
    }
}
