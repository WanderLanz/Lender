use crate::{FusedLender, Lender, Lending};

#[derive(Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Chunk<'s, T> {
    lender: &'s mut T,
    len: usize,
}
impl<'s, T> Chunk<'s, T> {
    pub(crate) fn new(lender: &'s mut T, len: usize) -> Self { Self { lender, len } }
}
impl<'lend, 's, T> Lending<'lend> for Chunk<'s, T>
where
    T: Lender,
{
    type Lend = <T as Lending<'lend>>::Lend;
}
impl<'s, T> Lender for Chunk<'s, T>
where
    T: Lender,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            self.lender.next()
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.lender.size_hint();
        (lower.min(self.len), upper.map(|x| x.min(self.len)))
    }
}
impl<'a, L> FusedLender for Chunk<'a, L> where L: FusedLender {}
