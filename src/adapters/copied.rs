use crate::{Lender, Lending};

pub struct Copied<L> {
    lender: L,
}
impl<L> Copied<L> {
    pub(crate) fn new(lender: L) -> Copied<L> { Copied { lender } }
}
impl<T, L> Iterator for Copied<L>
where
    L: Lender,
    T: Copy,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.lender.next().copied() }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
}
