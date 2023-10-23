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
pub fn once<'a, L: ?Sized + for<'all> Lending<'all>>(value: <L as Lending<'a>>::Lend) -> Once<'a, L> {
    Once { inner: Some(value) }
}

/// A lender that yields an element exactly once.
///
/// This `struct` is created by the [`once()`] function. See its documentation for more.
///
/// similar to [`core::iter::Once`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    inner: Option<<L as Lending<'a>>::Lend>,
}
impl<'a, L> Clone for Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
    <L as Lending<'a>>::Lend: Clone,
{
    fn clone(&self) -> Self {
        Once { inner: self.inner.clone() }
    }
}
impl<'a, L> fmt::Debug for Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
    <L as Lending<'a>>::Lend: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Once").field("inner", &self.inner).finish()
    }
}
impl<'a, 'lend, L> Lending<'lend> for Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L> Lender for Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        // SAFETY: 'a: 'lend
        self.inner
            .take()
            .map(|v| unsafe { core::mem::transmute::<<Self as Lending<'a>>::Lend, <Self as Lending<'_>>::Lend>(v) })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.inner.is_some() {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}

impl<'a, L> DoubleEndedLender for Once<'a, L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    #[inline]
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        self.next()
    }
}

impl<'a, L> ExactSizeLender for Once<'a, L> where L: ?Sized + for<'all> Lending<'all> {}

impl<'a, L> FusedLender for Once<'a, L> where L: ?Sized + for<'all> Lending<'all> {}
