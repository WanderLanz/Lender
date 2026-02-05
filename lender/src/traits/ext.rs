use fallible_iterator::{FallibleIterator, IntoFallibleIterator};

use crate::{FromFallibleIter, FromIntoFallibleIter, FromIntoIter, FromIter};

/// Extension trait adding to [`Iterator`] the method
/// [`into_lender`](IteratorExt::into_lender), which turns an [`Iterator`]
/// into a [`Lender`](crate::Lender) without allocation.
pub trait IteratorExt<I: Iterator> {
    /// Turn this [`Iterator`] into a [`Lender`](crate::Lender) without
    /// allocation.
    ///
    /// This method is a convenient entry point for
    /// [`from_iter`](crate::from_iter).
    fn into_lender(self) -> FromIter<I>;
}

impl<I: Iterator> IteratorExt<I> for I {
    #[inline]
    fn into_lender(self) -> FromIter<I> {
        crate::from_iter(self)
    }
}

/// Extension trait adding to [`IntoIterator`] the method
/// [`into_into_lender`](IntoIteratorExt::into_into_lender), which turns an
/// [`IntoIterator`] into a [`IntoLender`](crate::IntoLender) without
/// allocation.
pub trait IntoIteratorExt<I: IntoIterator> {
    /// Turn this [`IntoIterator`] into a [`IntoLender`](crate::IntoLender)
    /// without allocation.
    ///
    /// This method is a convenient entry point for
    /// [`from_into_iter`](crate::from_into_iter).
    fn into_into_lender(self) -> FromIntoIter<I>;
}

impl<I: IntoIterator> IntoIteratorExt<I> for I {
    #[inline]
    fn into_into_lender(self) -> FromIntoIter<I> {
        crate::from_into_iter(self)
    }
}

/// Extension trait adding to [`FallibleIterator`] the method
/// [`into_fallible_lender`](FallibleIteratorExt::into_fallible_lender), which
/// turns a [`FallibleIterator`] into a
/// [`FallibleLender`](crate::FallibleLender) without allocation.
pub trait FallibleIteratorExt<I: FallibleIterator> {
    /// Turn this [`FallibleIterator`] into a
    /// [`FallibleLender`](crate::FallibleLender) without allocation.
    ///
    /// This method is a convenient entry point for
    /// [`from_fallible_iter`](crate::from_fallible_iter).
    fn into_fallible_lender(self) -> FromFallibleIter<I>;
}

impl<I: FallibleIterator> FallibleIteratorExt<I> for I {
    #[inline]
    fn into_fallible_lender(self) -> FromFallibleIter<I> {
        crate::from_fallible_iter(self)
    }
}

/// Extension trait adding to [`IntoFallibleIterator`] the method
/// [`into_into_fallible_lender`][1], which turns an
/// [`IntoFallibleIterator`] into an [`IntoFallibleLender`] without
/// allocation.
///
/// [1]: IntoFallibleIteratorExt::into_into_fallible_lender
/// [`IntoFallibleLender`]: crate::IntoFallibleLender
pub trait IntoFallibleIteratorExt<I: IntoFallibleIterator> {
    /// Turn this [`IntoFallibleIterator`] into a
    /// [`IntoFallibleLender`](crate::IntoFallibleLender) without allocation.
    ///
    /// This method is a convenient entry point for
    /// [`from_into_fallible_iter`](crate::from_into_fallible_iter).
    fn into_into_fallible_lender(self) -> FromIntoFallibleIter<I>;
}

impl<I: IntoFallibleIterator> IntoFallibleIteratorExt<I> for I {
    #[inline]
    fn into_into_fallible_lender(self) -> FromIntoFallibleIter<I> {
        crate::from_into_fallible_iter(self)
    }
}
