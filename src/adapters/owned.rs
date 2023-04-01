use alloc::borrow::ToOwned;

use crate::{Lender, Lending};
pub struct Owned<L> {
    lender: L,
}
impl<L> Owned<L> {
    pub(crate) fn new(lender: L) -> Self { Self { lender } }
}
impl<T, L> Iterator for Owned<L>
where
    L: Lender,
    for<'all> <L as Lending<'all>>::Lend: ToOwned<Owned = T>,
{
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.lender.next().map(|x| x.to_owned()) }
}
