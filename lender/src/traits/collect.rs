use crate::{FallibleLender, Lend, Lender};
/// # Example
/// ```
/// # use lender::prelude::*;
/// struct MyStruct;
/// impl<L: IntoLender> FromLender<L> for MyStruct
/// where
///     L::Lender: for<'all> Lending<'all, Lend = &'all mut [u32]>,
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

/// The [`Lender`] version of [`core::iter::IntoIterator`].
pub trait IntoLender {
    type Lender: Lender;
    fn into_lender(self) -> <Self as IntoLender>::Lender;
}
impl<L: Lender> IntoLender for L {
    type Lender = L;
    #[inline]
    fn into_lender(self) -> L {
        self
    }
}
/// The [`Lender`] version of [`core::iter::Extend`].
pub trait ExtendLender<L: IntoLender> {
    fn extend_lender(&mut self, lender: L);
    /// Extends a collection with exactly one element.
    fn extend_lender_one(&mut self, item: Lend<'_, L::Lender>);
    /// Reserves capacity in a collection for the given number of additional elements.
    ///
    /// The default implementation does nothing.
    fn extend_lender_reserve(&mut self, additional: usize) {
        let _ = additional;
    }
}

/// The [`FallibleLender`] version of [`core::iter::IntoIterator`].
pub trait IntoFallibleLender {
    type Error;
    type FallibleLender: FallibleLender<Error = Self::Error>;
    fn into_fallible_lender(self) -> Self::FallibleLender;
}
impl<L: FallibleLender> IntoFallibleLender for L {
    type Error = L::Error;
    type FallibleLender = L;
    #[inline]
    fn into_fallible_lender(self) -> L {
        self
    }
}
