use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, IntoLender, Lender, Lending};

/// Creates a lender that yields an element exactly once.
///
/// lender equivalent to [`core::iter::once()`].
pub fn once<T>(value: T) -> Once<T> { Once { inner: Some(value).into_lender() } }

/// A lender that yields an element exactly once.
///
/// This `struct` is created by the [`once()`] function. See its documentation for more.
///
/// lender equivalent to [`core::iter::Once`].
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Once<T> {
    inner: crate::impls::option::OptionLender<T>,
}

impl<'lend, T> Lending<'lend> for Once<T> {
    type Lend = T;
}

impl<T> Lender for Once<T> {
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.inner.next() }

    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

impl<T> DoubleEndedLender for Once<T> {
    fn next_back(&mut self) -> Option<T> { self.inner.next_back() }
}

impl<T> ExactSizeLender for Once<T> {
    fn len(&self) -> usize { self.inner.len() }
}

impl<T> FusedLender for Once<T> {}
