use core::{iter::FusedIterator, marker::PhantomData};

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};

/// Iterator adapter for any Lender where Lend is `'static`,
/// allowing on-the-fly conversion into an iterator where Lending is no longer needed or inferred.
///
/// Implementing `Iterator` directly on any `Lender` where `Lend` is `'static` causes name conflicts, but might be possible in the future with specialization.
/// # Example
/// ```ignore
/// let mut vec = vec![1u8, 2, 3];
///
/// // windows_mut of vec...
///
/// let mut windows = windows_mut(vec, 2);
/// // ...is a Lender of &mut [u8], which is not owned
///
/// // let mut windows = windows.into_iterator(); // <-- this would not compile because &mut [u8] is not owned
///
/// let mut windows = windows.map(|x| x.to_vec());
/// // ...is a Lender of Vec<u8>, which is owned
///
/// let mut windows = windows.iter(); // <-- this compiles because Vec<u8> is owned
/// // ...is an Iterator of Vec<u8>
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<'l, L: 'l> {
    lender: L,
    _marker: PhantomData<&'l ()>,
}
impl<'l, L: 'l> Iter<'l, L> {
    pub(crate) fn new(lender: L) -> Iter<'l, L> { Iter { lender, _marker: PhantomData } }
}
impl<'l, L: 'l> Iterator for Iter<'l, L>
where
    L: Lender,
    for<'lend> <L as Lending<'lend>>::Lend: 'static,
{
    type Item = <L as Lending<'l>>::Lend;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> { unsafe { core::mem::transmute(self.lender.next()) } }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
}
impl<'l, L: 'l> DoubleEndedIterator for Iter<'l, L>
where
    L: DoubleEndedLender,
    for<'lend> <L as Lending<'lend>>::Lend: 'static,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> { unsafe { core::mem::transmute(self.lender.next_back()) } }
}
impl<'l, L: 'l> ExactSizeIterator for Iter<'l, L>
where
    L: ExactSizeLender,
    for<'lend> <L as Lending<'lend>>::Lend: 'static,
{
    #[inline]
    fn len(&self) -> usize { self.lender.len() }
}
impl<'l, L: 'l> FusedIterator for Iter<'l, L>
where
    L: FusedLender,
    for<'lend> <L as Lending<'lend>>::Lend: 'static,
{
}
