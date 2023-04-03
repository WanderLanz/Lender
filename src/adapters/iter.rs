use crate::{Lender, Lending};

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
/// let mut windows = windows.into_iterator(); // <-- this compiles because Vec<u8> is owned
/// // ...is an Iterator of Vec<u8>
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<L> {
    lender: L,
}
impl<L> Iter<L> {
    pub(crate) fn new(lender: L) -> Iter<L> { Iter { lender } }
}
impl<I, L> Iterator for Iter<L>
where
    L: Lender + for<'lend> Lending<'lend, Lend = I>,
    I: 'static,
{
    type Item = I;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.lender.next() }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
}
