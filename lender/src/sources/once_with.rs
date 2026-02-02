use core::fmt;

use crate::{
    DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending, higher_order::FnOnceHKA,
};

/// Creates a lender that lazily generates a value exactly once
/// by invoking the provided closure.
///
/// Note that functions passed to this function must be built
/// using the [`hrc!`](crate::hrc),
/// [`hrc_mut!`](crate::hrc_mut), or
/// [`hrc_once!`](crate::hrc_once) macro, which also checks for
/// covariance of the returned type. Circumventing the macro may
/// result in undefined behavior if the return type is not
/// covariant.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::once_with(0u8, hrc_once!(for<'all> |state: &'all mut u8| -> &'all mut u8 {
///     *state += 1;
///     state
/// }));
/// assert_eq!(lender.next(), Some(&mut 1));
/// assert_eq!(lender.next(), None);
/// ```
#[inline]
pub fn once_with<St, F>(state: St, f: F) -> OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    OnceWith { state, f: Some(f) }
}

/// A lender that yields a single element by applying the
/// provided closure.
///
/// This `struct` is created by the [`once_with()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct OnceWith<St, F> {
    state: St,
    f: Option<F>,
}

impl<St: Clone, F: Clone> Clone for OnceWith<St, F> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            f: self.f.clone(),
        }
    }
}

impl<St: fmt::Debug, F> fmt::Debug for OnceWith<St, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnceWith")
            .field("state", &self.state)
            .finish()
    }
}

impl<'lend, St, F> Lending<'lend> for OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    type Lend = <F as FnOnceHKA<'lend, &'lend mut St>>::B;
}

impl<St, F> Lender for OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.f.take().map(|f| f(&mut self.state))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.f.is_some() {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}

impl<St, F> DoubleEndedLender for OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.next()
    }
}

impl<St, F> ExactSizeLender for OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

impl<St, F> FusedLender for OnceWith<St, F> where F: for<'all> FnOnceHKA<'all, &'all mut St> {}
