use crate::{FromIntoIter, FromIter};

/// Extension trait adding to [`Iterator`] the method [`into_lend_iter`](IteratorExt::into_lend_iter),
/// which turns an [`Iterator`] into a [`Lender`](crate::Lender) without allocation.
pub trait IteratorExt<I: Iterator + Sized> {
    /// Turn this [`Iterator`] into a [`Lender`](crate::Lender) without allocation.
    ///
    /// This method is a convenient entry point for [`from_iter`](crate::from_iter).
    fn into_lend_iter(self) -> FromIter<I>;
}

impl<I: Iterator> IteratorExt<I> for I {
    fn into_lend_iter(self) -> FromIter<I> {
        crate::from_iter(self)
    }
}

pub trait IntoIteratorExt<I: IntoIterator + Sized> {
    /// Turn this [`IntoIterator`] into a [`IntoLender`](crate::IntoLender) without allocation.
    ///
    /// This method is a convenient entry point for [`from_into_iter`](crate::from_into_iter).
    fn into_into_lender(self) -> FromIntoIter<I>;
}

impl<I: Iterator> IntoIteratorExt<I> for I {
    fn into_into_lender(self) -> FromIntoIter<I> {
        crate::from_into_iter(self)
    }
}
