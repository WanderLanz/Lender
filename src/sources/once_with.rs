use crate::{higher_order::FnOnceHKA, DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};

/// Creates an lender that lazily generates a value exactly once by invoking
/// the provided closure.
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
pub fn once_with<St, F>(state: St, f: F) -> OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    OnceWith { state, f: Some(f) }
}

/// An iterator that yields a single element of type `A` by
/// applying the provided closure `F: FnOnce() -> A`.
///
/// This `struct` is created by the [`once_with()`] function.
/// See its documentation for more.
pub struct OnceWith<St, F> {
    state: St,
    f: Option<F>,
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
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
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
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        self.next()
    }
}

impl<St, F> ExactSizeLender for OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    #[inline]
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

impl<St, F> FusedLender for OnceWith<St, F> where F: for<'all> FnOnceHKA<'all, &'all mut St> {}
