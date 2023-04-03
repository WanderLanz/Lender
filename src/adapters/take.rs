use crate::{Lender, Lending};
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Take<L> {
    lender: L,
    n: usize,
}
impl<L> Take<L> {
    pub(crate) fn new(lender: L, n: usize) -> Take<L> { Take { lender, n } }
}
impl<'lend, L> Lending<'lend> for Take<L>
where
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L> Lender for Take<L>
where
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if self.n != 0 {
            self.n -= 1;
            self.lender.next()
        } else {
            None
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.lender.size_hint();
        (lower.min(self.n), upper.map(|x| x.min(self.n)))
    }
}
