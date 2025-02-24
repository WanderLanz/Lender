use core::fmt;

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending};

/// Creates a lender that yields an element exactly once.
///
/// similar to [`core::iter::once()`].
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut value = 42u32;
/// let mut o = lender::once::<lend!(&'lend mut u32)>(&mut value);
/// assert_eq!(o.next(), Some(&mut 42));
/// assert_eq!(o.next(), None);
/// ```
pub fn once<'a, L: ?Sized + for<'all> Lending<'all>>(value: Lend<'a, L>) -> Once<'a, L> {
    Once { inner: Some(value) }
}

/// A lender that yields an element exactly once.
///
/// This `struct` is created by the [`once()`] function.
///
/// similar to [`core::iter::Once`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    inner: Option<Lend<'a, L>>,
}
impl<'a, L> Clone for Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
    Lend<'a, L>: Clone,
{
    fn clone(&self) -> Self {
        Once { inner: self.inner.clone() }
    }
}
impl<'a, L> fmt::Debug for Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
    Lend<'a, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Once").field("inner", &self.inner).finish()
    }
}
impl<'lend, L> Lending<'lend> for Once<'_, L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L> Lender for Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend
        self.inner.take().map(|v| unsafe { core::mem::transmute::<Lend<'a, Self>, Lend<'_, Self>>(v) })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.inner.is_some() {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}

impl<L> DoubleEndedLender for Once<'_, L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.next()
    }
}

impl<L> ExactSizeLender for Once<'_, L> where L: ?Sized + for<'all> Lending<'all> {}

impl<L> FusedLender for Once<'_, L> where L: ?Sized + for<'all> Lending<'all> {}
