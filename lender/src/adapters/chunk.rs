use crate::{FusedLender, Lend, Lender, Lending};

#[derive(Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Chunk<'s, T> {
    lender: &'s mut T,
    len: usize,
}
impl<'s, T> Chunk<'s, T> {
    pub(crate) fn new(lender: &'s mut T, len: usize) -> Self {
        Self { lender, len }
    }
    pub fn into_inner(self) -> &'s mut T {
        self.lender
    }
    pub fn into_parts(self) -> (&'s mut T, usize) {
        (self.lender, self.len)
    }
}
impl<'lend, T> Lending<'lend> for Chunk<'_, T>
where
    T: Lender,
{
    type Lend = Lend<'lend, T>;
}
impl<T> Lender for Chunk<'_, T>
where
    T: Lender,
{
    fn next(&mut self) -> Option<Lend<'_, Self>> {
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
impl<L> FusedLender for Chunk<'_, L> where L: FusedLender {}
