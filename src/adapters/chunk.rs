use crate::{Lender, Lending};

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
}
