use fallible_iterator::{FallibleIterator, IntoFallibleIterator};

use crate::{
    FromFallibleIter, FromFallibleIterRef, FromIntoFallibleIter, FromIntoIter, FromIter,
    FromIterRef,
};

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
/// [`IntoIterator`] into an [`IntoLender`](crate::IntoLender) without
/// allocation.
pub trait IntoIteratorExt<I: IntoIterator> {
    /// Turn this [`IntoIterator`] into an [`IntoLender`](crate::IntoLender)
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
    /// Turn this [`IntoFallibleIterator`] into an
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

/// Extension trait adding to [`Iterator`] the method
/// [`into_ref_lender`](IteratorRefExt::into_ref_lender),
/// which turns an `Iterator<Item = T>` into a
/// [`Lender`](crate::Lender) with `Lend<'lend> = &'lend T`.
pub trait IteratorRefExt<I: Iterator> {
    /// Turn this [`Iterator`] into a [`Lender`](crate::Lender)
    /// that stores each element and lends a reference to it.
    ///
    /// This method is a convenient entry point for
    /// [`from_iter_ref`](crate::from_iter_ref).
    fn into_ref_lender(self) -> FromIterRef<I>;
}

impl<I: Iterator> IteratorRefExt<I> for I {
    #[inline]
    fn into_ref_lender(self) -> FromIterRef<I> {
        crate::from_iter_ref(self)
    }
}

/// Extension trait adding to [`FallibleIterator`] the method
/// [`into_fallible_ref_lender`](FallibleIteratorRefExt::into_fallible_ref_lender),
/// which turns a `FallibleIterator<Item = T>` into a
/// [`FallibleLender`](crate::FallibleLender) with `FallibleLend<'lend> = &'lend
/// T`.
pub trait FallibleIteratorRefExt<I: FallibleIterator> {
    /// Turn this [`FallibleIterator`] into a
    /// [`FallibleLender`](crate::FallibleLender) that stores
    /// each element and lends a reference to it.
    ///
    /// This method is a convenient entry point for
    /// [`from_fallible_iter_ref`](crate::from_fallible_iter_ref).
    fn into_fallible_ref_lender(self) -> FromFallibleIterRef<I>;
}

impl<I: FallibleIterator> FallibleIteratorRefExt<I> for I {
    #[inline]
    fn into_fallible_ref_lender(self) -> FromFallibleIterRef<I> {
        crate::from_fallible_iter_ref(self)
    }
}
