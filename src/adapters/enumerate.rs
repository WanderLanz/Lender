use crate::{Lender, Lending};
pub struct Enumerate<L> {
    lender: L,
    count: usize,
}
impl<L> Enumerate<L> {
    pub(crate) fn new(lender: L) -> Enumerate<L> { Enumerate { lender, count: 0 } }
}
impl<'lend, L> Lending<'lend> for Enumerate<L>
where
    L: Lender,
{
    type Lend = (usize, <L as Lending<'lend>>::Lend);
}
impl<L> Lender for Enumerate<L>
where
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        self.lender.next().map(|x| {
            let count = self.count;
            self.count += 1;
            (count, x)
        })
    }
}
