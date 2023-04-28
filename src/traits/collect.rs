use crate::{Lender, Lending};
/// Not stable. Currently just a specialization of [`From`] to use for [`Lender`] methods.
/// # Example
/// ```
/// use lender::*;
/// struct MyStruct;
/// impl<L: Lender> FromLender<L> for MyStruct
/// where
///     L: for<'all> Lending<'all, Lend = &'all mut [u32]>,
/// {
///     fn from_lender(lender: L) -> Self {
///         lender.for_each(|lend| drop(lend));
///         Self
///     }
/// }
/// ```
pub trait FromLender<L: Lender>: Sized {
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
/// Not stable. Currently just a specialization of [`From`] to use for [`Lender`] methods.
pub trait ExtendLender<T: IntoLender> {
    fn extend_lender(&mut self, lender: T);
    /// Extends a collection with exactly one element.
    fn extend_lender_one(&mut self, item: <T as Lending<'_>>::Lend);
    /// Reserves capacity in a collection for the given number of additional elements.
    ///
    /// The default implementation does nothing.
    fn extend_lender_reserve(&mut self, additional: usize) { let _ = additional; }
}
impl<T: IntoLender> ExtendLender<T> for ()
where
    T: for<'all> Lending<'all, Lend = ()>,
{
    fn extend_lender(&mut self, lender: T) { lender.into_lender().for_each(drop); }
    fn extend_lender_one(&mut self, item: <T as Lending<'_>>::Lend) { drop(item); }
}
