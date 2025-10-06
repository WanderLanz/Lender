use alloc::borrow::ToOwned;
use core::iter::FusedIterator;

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender};
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Owned<L> {
    lender: L,
}
impl<L> Owned<L> {
    pub(crate) fn new(lender: L) -> Self { Self { lender } }
    pub fn into_inner(self) -> L { self.lender }
}
impl<T, L> Iterator for Owned<L>
where
    L: Lender,
    for<'all> Lend<'all, L>: ToOwned<Owned = T>,
{
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.lender.next().map(|ref x| x.to_owned()) }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
}
impl<T, L> DoubleEndedIterator for Owned<L>
where
    L: DoubleEndedLender,
    for<'all> Lend<'all, L>: ToOwned<Owned = T>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> { self.lender.next_back().map(|ref x| x.to_owned()) }
}
impl<T, L> ExactSizeIterator for Owned<L>
where
    L: ExactSizeLender,
    for<'all> Lend<'all, L>: ToOwned<Owned = T>,
{
    fn len(&self) -> usize { self.lender.len() }
}
impl<T, L> FusedIterator for Owned<L>
where
    L: FusedLender,
    for<'all> Lend<'all, L>: ToOwned<Owned = T>,
{
}
impl<L> Default for Owned<L>
where
    L: Default,
{
    fn default() -> Self { Self::new(L::default()) }
}
