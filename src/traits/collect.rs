use crate::{Lender, Lending};
/// # Example
/// ```
/// # use lender::prelude::*;
/// struct MyStruct;
/// impl<L: IntoLender> FromLender<L> for MyStruct
/// where
///     L: for<'all> Lending<'all, Lend = &'all mut [u32]>,
/// {
///     fn from_lender(lender: L) -> Self {
///         lender.into_lender().for_each(|lend| drop(lend));
///         Self
///     }
/// }
/// ```
pub trait FromLender<L: IntoLender>: Sized {
    fn from_lender(lender: L) -> Self;
}
pub trait IntoLender: for<'all /* where Self: 'all */> Lending<'all> {
    type Lender: Lender + for<'all> Lending<'all, Lend = <Self as Lending<'all>>::Lend>;
    fn into_lender(self) -> <Self as IntoLender>::Lender;
}
impl<L: Lender> IntoLender for L {
    type Lender = L;
    #[inline]
    fn into_lender(self) -> L { self }
}
pub trait ExtendLender<L: IntoLender> {
    fn extend_lender(&mut self, lender: L);
    /// Extends a collection with exactly one element.
    fn extend_lender_one(&mut self, item: <L as Lending<'_>>::Lend);
    /// Reserves capacity in a collection for the given number of additional elements.
    ///
    /// The default implementation does nothing.
    fn extend_lender_reserve(&mut self, additional: usize) { let _ = additional; }
}
