use crate::{FallibleLend, FallibleLender, Lend, Lender};

/// A trait for creating a value from a [`Lender`].
///
/// # Examples
/// ```
/// # use lender::prelude::*;
/// struct MyStruct;
/// impl<L: IntoLender> FromLender<L> for MyStruct
/// where
///     L::Lender: for<'all> Lending<'all, Lend = &'all mut [i32]>,
/// {
///     fn from_lender(lender: L) -> Self {
///         lender.into_lender().for_each(|lend| drop(lend));
///         Self
///     }
/// }
/// ```
pub trait FromLender<L: IntoLender>: Sized {
    /// Creates a value from a [`Lender`].
    fn from_lender(lender: L) -> Self;
}

/// A trait for creating a value from a [`FallibleLender`].
///
/// This is the fallible counterpart to [`FromLender`].
///
/// # Examples
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
    /// Creates a value from a [`FallibleLender`],
    /// returning an error if the lender produces one.
    fn from_fallible_lender(lender: L) -> Result<Self, L::Error>;
}

/// Conversion into a [`Lender`].
///
/// This is the [`Lender`] version of [`core::iter::IntoIterator`].
///
/// Every [`Lender`] implements `IntoLender` for itself (returning `self`).
pub trait IntoLender {
    /// The lender type that this type converts into.
    type Lender: Lender;
    /// Converts this type into a [`Lender`].
    fn into_lender(self) -> <Self as IntoLender>::Lender;
}

impl<L: Lender> IntoLender for L {
    type Lender = L;
    #[inline(always)]
    fn into_lender(self) -> L {
        self
    }
}

/// The [`Lender`] version of [`core::iter::Extend`].
pub trait ExtendLender<L: IntoLender> {
    /// Extends a collection with the contents of a lender.
    fn extend_lender(&mut self, lender: L);
    /// Extends a collection with exactly one element.
    fn extend_lender_one(&mut self, item: Lend<'_, L::Lender>);
    /// Reserves capacity in a collection for the given number
    /// of additional elements.
    ///
    /// The default implementation does nothing.
    #[inline(always)]
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

    /// Reserves capacity in a collection for the given number
    /// of additional elements.
    ///
    /// The default implementation does nothing.
    #[inline(always)]
    fn extend_fallible_lender_reserve(&mut self, additional: usize) {
        let _ = additional;
    }
}

/// Conversion into a [`FallibleLender`].
///
/// This is the [`FallibleLender`] version of [`core::iter::IntoIterator`].
///
/// By implementing `IntoFallibleLender` for a type, you define how it will be
/// converted to a fallible lender.
///
/// Every [`FallibleLender`] implements `IntoFallibleLender` for itself
/// (returning `self`).
pub trait IntoFallibleLender {
    /// The error type of the resulting fallible lender.
    type Error;
    /// The fallible lender type that this type converts into.
    type FallibleLender: FallibleLender<Error = Self::Error>;
    /// Converts this type into a [`FallibleLender`].
    fn into_fallible_lender(self) -> Self::FallibleLender;
}

impl<L: FallibleLender> IntoFallibleLender for L {
    type Error = L::Error;
    type FallibleLender = L;
    #[inline(always)]
    fn into_fallible_lender(self) -> L {
        self
    }
}
