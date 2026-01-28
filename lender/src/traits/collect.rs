use crate::{FallibleLend, FallibleLender, Lend, Lender};

/// A trait for creating a value from a [`Lender`].
///
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

/// A trait for creating a value from a [`FallibleLender`].
///
/// This is the fallible counterpart to [`FromLender`].
///
/// # Example
/// ```
/// # use lender::prelude::*;
/// # use std::convert::Infallible;
/// struct MyVec(Vec<i32>);
///
/// impl<L: IntoFallibleLender<Error = Infallible>> FromFallibleLender<L> for MyVec
/// where
///     L::FallibleLender: for<'all> FallibleLending<'all, Lend = i32>,
/// {
///     fn from_fallible_lender(lender: L) -> Result<Self, Infallible> {
///         let mut vec = Vec::new();
///         lender.into_fallible_lender().for_each(|x| {
///             vec.push(x);
///             Ok(())
///         })?;
///         Ok(MyVec(vec))
///     }
/// }
/// ```
pub trait FromFallibleLender<L: IntoFallibleLender>: Sized {
    fn from_fallible_lender(lender: L) -> Result<Self, L::Error>;
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

/// The [`FallibleLender`] version of [`core::iter::Extend`].
///
/// This is the fallible counterpart to [`ExtendLender`].
pub trait ExtendFallibleLender<L: IntoFallibleLender> {
    /// Extends a collection with elements from a fallible lender.
    ///
    /// Returns an error if the lender produces an error during iteration.
    fn extend_fallible_lender(&mut self, lender: L) -> Result<(), L::Error>;

    /// Extends a collection with exactly one element.
    fn extend_fallible_lender_one(&mut self, item: FallibleLend<'_, L::FallibleLender>);

    /// Reserves capacity in a collection for the given number of additional elements.
    ///
    /// The default implementation does nothing.
    fn extend_fallible_lender_reserve(&mut self, additional: usize) {
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
