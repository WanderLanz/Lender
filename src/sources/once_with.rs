use core::fmt;

use crate::{hkts::FnOnceHK, DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};

/// Creates a lender that lazily generates a value exactly once by invoking
/// the provided closure.
///
/// lender equivalent to [`core::iter::once_with()`].
#[inline]
pub fn once_with<F: for<'all> FnOnceHK<'all>>(gen: F) -> OnceWith<F> { OnceWith { gen: Some(gen) } }

/// A lender that yields a single element of type `A` by
/// applying the provided closure `F: FnOnce() -> A`.
///
/// This `struct` is created by the [`once_with()`] function.
/// See its documentation for more.
///
/// lender equivalent to [`core::iter::OnceWith`].

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct OnceWith<F> {
    gen: Option<F>,
}

impl<F> fmt::Debug for OnceWith<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.gen.is_some() {
            f.write_str("OnceWith(Some(_))")
        } else {
            f.write_str("OnceWith(None)")
        }
    }
}

impl<'lend, F: for<'all> FnOnceHK<'all>> Lending<'lend> for OnceWith<F> {
    type Lend = <F as FnOnceHK<'lend>>::B;
}

impl<F: for<'all> FnOnceHK<'all>> Lender for OnceWith<F> {
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        let f = self.gen.take()?;
        Some(f())
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.gen.iter().size_hint() }
}

impl<F: for<'all> FnOnceHK<'all>> DoubleEndedLender for OnceWith<F> {
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.next() }
}

impl<F: for<'all> FnOnceHK<'all>> ExactSizeLender for OnceWith<F> {
    fn len(&self) -> usize { self.gen.iter().len() }
}

impl<F: for<'all> FnOnceHK<'all>> FusedLender for OnceWith<F> {}
