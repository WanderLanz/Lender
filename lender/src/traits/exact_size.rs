use crate::*;

/// Documentation is incomplete. Refer to [`core::iter::ExactSizeIterator`] for more information
pub trait ExactSizeLender: Lender {
    #[inline]
    fn len(&self) -> usize {
        let (lower, upper) = self.size_hint();
        assert_eq!(upper, Some(lower));
        lower
    }
    #[inline]
    fn is_empty(&self) -> bool { self.len() == 0 }
}
impl<L: ExactSizeLender + ?Sized> ExactSizeLender for &mut L {
    fn len(&self) -> usize { (**self).len() }
    fn is_empty(&self) -> bool { (**self).is_empty() }
}

/// A lender that knows if it has additional elements.
///
/// Many [`Iterator`]s don't have a use for this kind of trait, but [`Lender`]s do because of lending requirements.
///
/// The [`has_next`](HasNext::has_next) method has a default implementation relying on [`size_hint`](Lender::size_hint), so you should
/// override the implementation if [`size_hint`](Lender::size_hint) is not accurate.
///
/// Note that this trait is a safe trait and as such does *not* and *cannot*
/// guarantee that the returned assertion is correct. This means that `unsafe`
/// code **must not** rely on the correctness of [`has_next`](HasNext::has_next).
pub trait HasNext: Lender {
    /// Returns true if there are more lends, and false otherwise.
    fn has_next(&mut self) -> bool { self.size_hint().0 > 0 }
}

impl<L: ?Sized> HasNext for L
where
    L: ExactSizeLender,
{
    fn has_next(&mut self) -> bool { !self.is_empty() }
}
