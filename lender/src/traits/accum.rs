use crate::Lender;
/// Sums lends of a [`Lender`] into a single Self.
///
/// This trait is similar to [`core::iter::Sum`], but for [`Lender`]s.
///
/// # Example
/// ```rust
/// # use std::borrow::ToOwned;
/// # use lender::{prelude::*, SumLender};
/// struct U32Sum(pub u32);
/// impl<'lend> Lending<'lend> for U32Sum {
///    type Lend = &'lend u32;
/// }
/// impl<L: Lender> SumLender<L> for U32Sum
/// where
///     for<'all> L: Lending<'all, Lend = &'all u32>,
/// {
///     fn sum_lender(lender: L) -> Self {
///         U32Sum(lender.fold(0, |acc, x| acc + *x))
///     }
/// }
/// let e = lender::empty::<lend!(&'lend u32)>();
/// assert_eq!(U32Sum::sum_lender(e).0, 0u32);
/// ```
pub trait SumLender<L: Lender>: Sized {
    fn sum_lender(lender: L) -> Self;
}

/// Documentation is incomplete. Refer to [`core::iter::Product`] for more information
pub trait ProductLender<L: Lender>: Sized {
    fn product_lender(lender: L) -> Self;
}
