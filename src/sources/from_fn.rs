use core::fmt;

use crate::{higher_order::FnMutHKAOpt, Lender, Lending};

/// Creates a lender from a state and a closure `F: FnMut(&mut St) -> Option<T>`.
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

/// An lender where each iteration calls the provided closure `F: FnMut(&mut St) -> Option<T>`.
///
/// This `struct` is created by the [`from_fn()`] function.
/// See its documentation for more.
#[derive(Clone)]
pub struct FromFn<St, F> {
    state: St,
    f: F,
}

impl<St: fmt::Debug, F> fmt::Debug for FromFn<St, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromFn").field("state", &self.state).finish()
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
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        (self.f)(&mut self.state)
    }
}
