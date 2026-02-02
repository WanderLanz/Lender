use core::{iter::FusedIterator, marker::PhantomData};

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender};

/// [`Iterator`] adapter for any [`Lender`] where multiple [`Lend`]s can exist at
/// a time, allowing on-the-fly conversion into an iterator where
/// [`Lending`](crate::Lending) is no longer needed or inferred.
///
/// Implementing [`Iterator`] directly on any [`Lender`] causes name conflicts,
/// but might be possible in the future with specialization.
///
/// # Example
///
/// ```rust
/// use lender::prelude::*;
///
/// let mut data = [1u8, 2, 3];
///
/// // windows_mut is a Lender of &mut [u8] (non-owned)
/// let windows = lender::windows_mut(&mut data, 2);
///
/// // Map to owned values, then convert to Iterator
/// let mapped = windows.map(hrc_mut!(
///     for<'all> |w: &'all mut [u8]| -> Vec<u8> {
///         w.to_vec()
///     }
/// ));
/// let result: Vec<Vec<u8>> = mapped.iter().collect();
/// assert_eq!(result, vec![vec![1, 2], vec![2, 3]]);
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<'this, L: 'this> {
    pub(crate) lender: L,
    pub(crate) _marker: PhantomData<&'this ()>,
}

impl<'this, L: 'this> Iter<'this, L> {
    #[inline(always)]
    pub(crate) fn new(lender: L) -> Iter<'this, L> {
        Iter {
            lender,
            _marker: PhantomData,
        }
    }

    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }
}

impl<'this, L: 'this> Iterator for Iter<'this, L>
where
    L: Lender,
    for<'all> Lend<'all, L>: 'this,
{
    type Item = Lend<'this, L>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: for<'all> Lend<'all, L>: 'this
        unsafe {
            core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(self.lender.next())
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}

impl<'this, L: 'this> DoubleEndedIterator for Iter<'this, L>
where
    L: DoubleEndedLender,
    for<'all> Lend<'all, L>: 'this,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        // SAFETY: for<'all> Lend<'all, L>: 'this
        unsafe {
            core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(
                self.lender.next_back(),
            )
        }
    }
}

impl<'this, L: 'this> ExactSizeIterator for Iter<'this, L>
where
    L: ExactSizeLender,
    for<'all> Lend<'all, L>: 'this,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }
}

impl<'this, L: 'this> FusedIterator for Iter<'this, L>
where
    L: FusedLender,
    for<'all> Lend<'all, L>: 'this,
{
}
