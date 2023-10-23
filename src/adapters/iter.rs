use core::{iter::FusedIterator, marker::PhantomData};

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};

/// Iterator adapter for any Lender where multiple Lends can exist at a time,
/// allowing on-the-fly conversion into an iterator where Lending is no longer needed or inferred.
///
/// Implementing `Iterator` directly on any `Lender` causes name conflicts, but might be possible in the future with specialization.
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
pub struct Iter<'this, L: 'this> {
    lender: L,
    _marker: PhantomData<&'this ()>,
}
impl<'this, L: 'this> Iter<'this, L> {
    pub(crate) fn new(lender: L) -> Iter<'this, L> {
        Iter { lender, _marker: PhantomData }
    }
}
impl<'this, L: 'this> Iterator for Iter<'this, L>
where
    L: Lender,
    for<'all> <L as Lending<'all>>::Lend: 'this,
{
    type Item = <L as Lending<'this>>::Lend;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: for<'all> <L as Lending<'all>>::Lend: 'this
        unsafe {
            core::mem::transmute::<Option<<L as Lending<'_>>::Lend>, Option<<L as Lending<'this>>::Lend>>(self.lender.next())
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}
impl<'this, L: 'this> DoubleEndedIterator for Iter<'this, L>
where
    L: DoubleEndedLender,
    for<'all> <L as Lending<'all>>::Lend: 'this,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        // SAFETY: for<'all> <L as Lending<'all>>::Lend: 'this
        unsafe {
            core::mem::transmute::<Option<<L as Lending<'_>>::Lend>, Option<<L as Lending<'this>>::Lend>>(
                self.lender.next_back(),
            )
        }
    }
}
impl<'this, L: 'this> ExactSizeIterator for Iter<'this, L>
where
    L: ExactSizeLender,
    for<'all> <L as Lending<'all>>::Lend: 'this,
{
    #[inline]
    fn len(&self) -> usize {
        self.lender.len()
    }
}
impl<'this, L: 'this> FusedIterator for Iter<'this, L>
where
    L: FusedLender,
    for<'all> <L as Lending<'all>>::Lend: 'this,
{
}
