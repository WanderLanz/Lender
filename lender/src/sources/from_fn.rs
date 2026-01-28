use core::{fmt, marker::PhantomData};

use crate::{
    higher_order::{FnMutHKAOpt, FnMutHKAResOpt},
    FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending,
};

/// Creates a lender from a state and a closure `F: FnMut(&mut St) -> Option<T>`.
///
/// Note that functions passed to this function must be built using the
/// [`hrc!`](crate::hrc) or [`hrc_mut!`](crate::hrc_mut) macro, which also
/// checks for covariance of the returned type. Circumventing the macro may
/// result in undefined behavior if the return type is not covariant.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::from_fn(0u8, hrc_mut!(for<'all> |state: &'all mut u8| -> Option<&'all mut u8> {
///     if *state < 10 {
///         *state += 1;
///         Some(state)
///     } else {
///         None
///     }
/// }));
/// assert_eq!(lender.next(), Some(&mut 1));
/// ```
pub fn from_fn<St, F>(state: St, f: F) -> FromFn<St, F>
where
    F: for<'all> FnMutHKAOpt<'all, &'all mut St>,
{
    FromFn { state, f }
}

/// A lender where each iteration calls the provided closure `F: FnMut(&mut St) -> Option<T>`.
///
/// This `struct` is created by the [`from_fn()`] function.
///
#[derive(Clone)]
pub struct FromFn<St, F> {
    state: St,
    f: F,
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
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        (self.f)(&mut self.state)
    }
}

/// Creates a fallible lender from a state and a closure `F: FnMut(&mut St) -> Result<Option<T>, E>`.
///
/// Note that functions passed to this function must be built using the
/// [`hrc!`](crate::hrc) or [`hrc_mut!`](crate::hrc_mut) macro, which also
/// checks for covariance of the returned type. Circumventing the macro may
/// result in undefined behavior if the return type is not covariant.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// # use std::io::Error;
/// let mut lender = lender::from_fallible_fn::<_, Error, _>(
///     0u8,
///     hrc_mut!(for<'all> |state: &'all mut u8| -> Result<Option<&'all mut u8>, Error> {
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
pub fn from_fallible_fn<St, E, F>(state: St, f: F) -> FromFallibleFn<St, E, F>
where
    F: for<'all> FnMutHKAResOpt<'all, &'all mut St, E>,
{
    FromFallibleFn {
        state,
        f,
        _marker: PhantomData,
    }
}

// An lender where each iteration calls the provided closure `F: FnMut(&mut St) -> Result<Option<T>, E>`.
///
/// This `struct` is created by the [`from_fallible_fn()`] function.
///
#[derive(Clone)]
pub struct FromFallibleFn<St, E, F> {
    state: St,
    f: F,
    _marker: PhantomData<E>,
}

impl<St: fmt::Debug, E, F> fmt::Debug for FromFallibleFn<St, E, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromFallibleFn")
            .field("state", &self.state)
            .finish()
    }
}

impl<'lend, St, E, F> FallibleLending<'lend> for FromFallibleFn<St, E, F>
where
    F: for<'all> FnMutHKAResOpt<'all, &'all mut St, E>,
{
    type Lend = <F as FnMutHKAResOpt<'lend, &'lend mut St, E>>::B;
}

impl<St, E, F> FallibleLender for FromFallibleFn<St, E, F>
where
    F: for<'all> FnMutHKAResOpt<'all, &'all mut St, E>,
{
    type Error = E;
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        (self.f)(&mut self.state)
    }
}
