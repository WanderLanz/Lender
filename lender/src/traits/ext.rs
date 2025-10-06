use crate::{FromIntoIter, FromIter};

/// Extension trait adding to [`Iterator`] the method [`into_lender`](IteratorExt::into_lender),
/// which turns an [`Iterator`] into a [`Lender`](crate::Lender) without allocation.
pub trait IteratorExt<I: Iterator> {
    /// Turn this [`Iterator`] into a [`Lender`](crate::Lender) without allocation.
    ///
    /// This method is a convenient entry point for [`from_iter`](crate::from_iter).
    fn into_lender(self) -> FromIter<I>;
}

impl<I: Iterator> IteratorExt<I> for I {
    fn into_lender(self) -> FromIter<I> { crate::from_iter(self) }
}

/// Extension trait adding to [`IntoIterator`] the method [`into_into_lender`](IntoIteratorExt::into_into_lender),
/// which turns an [`IntoIterator`] into a [`IntoLender`](crate::IntoLender) without allocation.
pub trait IntoIteratorExt<I: IntoIterator> {
    /// Turn this [`IntoIterator`] into a [`IntoLender`](crate::IntoLender) without allocation.
    ///
    /// This method is a convenient entry point for [`from_into_iter`](crate::from_into_iter).
    fn into_into_lender(self) -> FromIntoIter<I>;
}

impl<I: IntoIterator> IntoIteratorExt<I> for I {
    fn into_into_lender(self) -> FromIntoIter<I> { crate::from_into_iter(self) }
}
