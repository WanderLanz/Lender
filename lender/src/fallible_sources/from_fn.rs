use core::{fmt, marker::PhantomData};

use crate::{Covar, FallibleLend, FallibleLender, FallibleLending, higher_order::FnMutHKAResOpt};

/// Creates a fallible lender from a state and a closure
/// `F: FnMut(&mut St) -> Result<Option<T>, E>`.
///
/// Note that functions passed to this function must be built
/// using the [`covar!`](crate::covar) or [`covar_mut!`](crate::covar_mut)
/// macros, which also checks for covariance of the returned type.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// # use std::io::Error;
/// let mut lender = lender::from_fallible_fn::<_, Error, _>(
///     0,
///     covar_mut!(for<'all> |state: &'all mut i32| -> Result<Option<&'all mut i32>, Error> {
///         if *state < 3 {
///             *state += 1;
///             Ok(Some(state))
///         } else {
///             Ok(None)
///         }
///     })
/// );
/// assert_eq!(lender.next().unwrap(), Some(&mut 1));
/// ```
#[inline]
pub fn from_fn<St, E, F>(state: St, f: Covar<F>) -> FromFn<St, E, F>
where
    F: for<'all> FnMutHKAResOpt<'all, &'all mut St, E>,
{
    FromFn {
        state,
        f,
        _marker: PhantomData,
    }
}

/// A lender where each iteration calls the provided closure
/// `F: FnMut(&mut St) -> Result<Option<T>, E>`.
///
/// This `struct` is created by the [`from_fallible_fn()`](crate::from_fallible_fn) function.
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FromFn<St, E, F> {
    state: St,
    f: Covar<F>,
    _marker: PhantomData<fn() -> E>,
}

impl<St: fmt::Debug, E, F> fmt::Debug for FromFn<St, E, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromFallibleFn")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl<'lend, St, E, F> FallibleLending<'lend> for FromFn<St, E, F>
where
    F: for<'all> FnMutHKAResOpt<'all, &'all mut St, E>,
{
    type Lend = <F as FnMutHKAResOpt<'lend, &'lend mut St, E>>::B;
}

impl<St, E, F> FallibleLender for FromFn<St, E, F>
where
    F: for<'all> FnMutHKAResOpt<'all, &'all mut St, E>,
{
    type Error = E;
    // SAFETY: the lend is the return type of F, whose covariance
    // has been checked at Covar construction time.
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        (self.f.as_inner_mut())(&mut self.state)
    }
}
