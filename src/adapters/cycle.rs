use crate::{Lender, Lending};

pub struct Cycle<L> {
    orig: L,
    lender: L,
}
impl<L> Cycle<L>
where
    L: Clone,
{
    pub(crate) fn new(lender: L) -> Cycle<L> { Cycle { orig: lender.clone(), lender } }
}
impl<'lend, L> Lending<'lend> for Cycle<L>
where
    L: Clone + Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L> Lender for Cycle<L>
where
    L: Clone + Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        let reborrow = unsafe { &mut *(self as *mut Self) };
        if let x @ Some(_) = reborrow.lender.next() {
            return x;
        }
        self.lender = self.orig.clone();
        self.lender.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.orig.size_hint() {
            h @ (0, Some(0)) => h,
            (0, _) => (0, None),
            _ => (usize::MAX, None),
        }
    }
}
