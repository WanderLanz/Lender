use crate::{Lender, Lending};
pub struct Fuse<L> {
    lender: L,
    flag: bool,
}
impl<L> Fuse<L> {
    pub(crate) fn new(lender: L) -> Fuse<L> { Fuse { lender, flag: false } }
}
impl<'lend, L> Lending<'lend> for Fuse<L>
where
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L> Lender for Fuse<L>
where
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if !self.flag {
            if let x @ Some(_) = self.lender.next() {
                return x;
            }
            self.flag = true;
        }
        None
    }
}
