use crate::{Lender, Lending};

pub struct Cloned<L> {
    lender: L,
}
impl<L> Cloned<L> {
    pub(crate) fn new(lender: L) -> Cloned<L> { Cloned { lender } }
}
impl<T, L> Iterator for Cloned<L>
where
    L: Lender,
    T: Clone,
    L: for<'all> Lending<'all, Lend = &'all T>,
{
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.lender.next().cloned() }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
}
