use core::fmt;

use crate::{Covar, Lend, Lender, Lending, higher_order::FnMutHKAOpt};

/// Creates a lender from a state and a closure
/// `F: FnMut(&mut St) -> Option<T>`.
///
/// Note that functions passed to this function must be built
/// using the [`covar!`](crate::covar) or [`covar_mut!`](crate::covar_mut)
/// macro, which also checks for covariance of the returned type.
/// Circumventing the macro may result in undefined behavior if
/// the return type is not covariant.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::from_fn(0u8, covar_mut!(for<'all> |state: &'all mut u8| -> Option<&'all mut u8> {
///     if *state < 10 {
///         *state += 1;
///         Some(state)
///     } else {
///         None
///     }
/// }));
/// assert_eq!(lender.next(), Some(&mut 1));
/// ```
#[inline]
pub fn from_fn<St, F>(state: St, f: Covar<F>) -> FromFn<St, F>
where
    F: for<'all> FnMutHKAOpt<'all, &'all mut St>,
{
    FromFn { state, f }
}

/// A lender where each iteration calls the provided closure
/// `F: FnMut(&mut St) -> Option<T>`.
///
/// This `struct` is created by the [`from_fn()`] function.
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FromFn<St, F> {
    state: St,
    f: Covar<F>,
}

impl<St: fmt::Debug, F> fmt::Debug for FromFn<St, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromFn")
            .field("state", &self.state)
            .finish()
    }
}

impl<'lend, St, F> Lending<'lend> for FromFn<St, F>
where
    F: for<'all> FnMutHKAOpt<'all, &'all mut St>,
{
    type Lend = <F as FnMutHKAOpt<'lend, &'lend mut St>>::B;
}

impl<St, F> Lender for FromFn<St, F>
where
    F: for<'all> FnMutHKAOpt<'all, &'all mut St>,
{
    // SAFETY: the lend is the return type of F, whose covariance
    // has been checked at Covar construction time.
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        (self.f.as_inner_mut())(&mut self.state)
    }
}
