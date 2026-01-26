mod accum;
mod collect;
mod double_ended;
mod exact_size;
mod ext;
mod fallible_lender;
mod lender;
mod marker;

pub use self::{
    accum::{ProductFallibleLender, ProductLender, SumFallibleLender, SumLender},
    collect::{ExtendLender, FromLender, IntoFallibleLender, IntoLender},
    double_ended::{DoubleEndedFallibleLender, DoubleEndedLender},
    exact_size::{ExactSizeFallibleLender, ExactSizeLender},
    ext::{FallibleIteratorExt, IntoFallibleIteratorExt, IntoIteratorExt, IteratorExt},
    fallible_lender::{FallibleLend, FallibleLender, FallibleLending},
    lender::{Lend, Lender, Lending},
    marker::{FusedFallibleLender, FusedLender},
};

/// Marker trait for tuple lends, used by [`Lender::unzip()`].
pub trait TupleLend<'a> {
    type First: 'a;
    type Second: 'a;
    fn tuple_lend(self) -> (Self::First, Self::Second);
}
impl<'a, A: 'a, B: 'a> TupleLend<'a> for (A, B) {
    type First = A;
    type Second = B;
    #[inline(always)]
    fn tuple_lend(self) -> (Self::First, Self::Second) {
        (self.0, self.1)
    }
}
impl<'a, A, B> TupleLend<'a> for &'a (A, B) {
    type First = &'a A;
    type Second = &'a B;
    #[inline(always)]
    fn tuple_lend(self) -> (Self::First, Self::Second) {
        (&self.0, &self.1)
    }
}
impl<'a, A, B> TupleLend<'a> for &'a mut (A, B) {
    type First = &'a mut A;
    type Second = &'a mut B;
    #[inline(always)]
    fn tuple_lend(self) -> (Self::First, Self::Second) {
        (&mut self.0, &mut self.1)
    }
}

/// Internal struct used to implement [`lend!`], do not use directly.
#[doc(hidden)]
pub struct DynLendShunt<T: ?Sized>(pub T);

impl<'lend, T: ?Sized + for<'all> DynLend<'all>> Lending<'lend> for DynLendShunt<T> {
    type Lend = <T as DynLend<'lend>>::Lend;
}

/// Internal trait used to implement [`lend!`], do not use directly.
#[doc(hidden)]
pub trait DynLend<'lend> {
    type Lend;
}

/// Use lifetime `'lend` within type `$T` to create an `impl for<'lend> Lending<'lend, Lend = $T>`.
///
/// Uses a bug in the borrow checker which allows dyn objects to implement impossible traits.
///
/// # Safety Note
///
/// When using this with [`lend_iter`](crate::lend_iter), the lend type `$T` **must be covariant
/// in `'lend`**. Using an invariant type (e.g., `&'lend Cell<&'lend T>`) leads to undefined
/// behavior. Use [`covariant_lend!`] for compile-time covariance checking.
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut empty = lender::empty::<lend!(&'lend mut [u32])>(); // <- same Lending signature as a WindowsMut over u32
/// let _: Option<&mut [u32]> = empty.next(); // => None
/// ```
#[macro_export]
macro_rules! lend {
    ($T:ty) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = $T>>
    };
}

/// Implement the covariance check method for a [`Lender`] impl with a concrete
/// [`Lend`](Lending::Lend) type.
///
/// This macro must be invoked inside `impl Lender for T` blocks where the
/// [`Lend`](Lending::Lend) type is concrete (not defined in terms of another
/// lender's [`Lend`](Lending::Lend) type). It expands to the
/// `_check_covariance` method implementation with body `{ lend }`, which only
/// compiles if the `Lend` type is covariant in its lifetime.
///
/// For adapters that delegate to underlying lenders, use
/// [`inherit_covariance!`](crate::inherit_covariance!) instead.
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
///
/// struct RefLender<'a, T>(&'a [T], usize);
///
/// impl<'lend, T> Lending<'lend> for RefLender<'_, T> {
///     type Lend = &'lend T;  // Concrete covariant type
/// }
///
/// impl<T> Lender for RefLender<'_, T> {
///     check_covariance!();
///
///     fn next(&mut self) -> Option<Lend<'_, Self>> {
///         if self.1 < self.0.len() {
///             let i = self.1;
///             self.1 += 1;
///             Some(&self.0[i])
///         } else {
///             None
///         }
///     }
/// }
/// ```
///
/// # Compile-time Error for Invariant Types
///
/// If your `Lend` type is invariant (e.g., `&'lend Cell<&'lend T>`), this macro
/// will cause a compile error, preventing undefined behavior.
#[macro_export]
macro_rules! check_covariance {
    () => {
        unsafe fn _check_covariance<'long: 'short, 'short>(
            lend: <Self as $crate::Lending<'long>>::Lend,
        ) -> <Self as $crate::Lending<'short>>::Lend {
            lend
        }
    };
}

/// Implement the covariance check method for adapter [`Lender`] impls.
///
/// Use this macro for adapters whose `Lend` type is defined in terms of another
/// lender's `Lend` type (e.g., `type Lend = Lend<'lend, L>`). The covariance is
/// inherited from the underlying lender, which must implement `_check_covariance`.
///
/// For lenders with concrete `Lend` types, use [`check_covariance!`] instead.
///
/// # Safety
///
/// This macro uses transmute internally. It is safe because the underlying lender `L`
/// is required to implement `_check_covariance`, which ensures `L`'s `Lend` type is
/// covariant. Since this adapter's `Lend` type is derived from `L`'s, the covariance
/// is transitively guaranteed.
#[macro_export]
macro_rules! inherit_covariance {
    () => {
        unsafe fn _check_covariance<'long: 'short, 'short>(
            lend: <Self as $crate::Lending<'long>>::Lend,
        ) -> <Self as $crate::Lending<'short>>::Lend {
            // SAFETY: Covariance is inherited from the underlying Lender,
            // which is required to implement _check_covariance, ensuring
            // their Lend type is covariant.
            unsafe { core::mem::transmute(lend) }
        }
    };
}

/// Define a lending type with compile-time covariance checking.
///
/// This macro creates a struct that implements [`Lending`] and includes a compile-time
/// assertion that the lend type is covariant in its lifetime. Use this when you need
/// to ensure type safety with [`lend_iter`](crate::lend_iter).
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
///
/// // Define a covariance-checked lending type
/// lender::covariant_lend!(RefU32 = &'lend u32);
///
/// let data = [1u32, 2, 3];
/// let mut lender = lender::lend_iter::<'_, RefU32, _>(data.iter());
/// let item: &u32 = lender.next().unwrap();
/// ```
///
/// # Compile-time Error for Invariant Types
///
/// The following will fail to compile because `Cell<Option<&'lend String>>` is
/// invariant in `'lend`:
///
/// ```rust,compile_fail
/// use std::cell::Cell;
/// use lender::prelude::*;
///
/// // This fails to compile - Cell makes the type invariant!
/// lender::covariant_lend!(InvariantLend = &'lend Cell<Option<&'lend String>>);
/// ```
#[macro_export]
macro_rules! covariant_lend {
    ($name:ident = $T:ty) => {
        /// A lending type with compile-time covariance checking.
        #[derive(Clone, Copy, Debug, Default)]
        struct $name;

        impl<'lend> $crate::Lending<'lend> for $name {
            type Lend = $T;
        }

        // Covariance check: this const only compiles if $T is covariant in 'lend.
        // The check works by attempting to assign `Option<Lend<'long>>` to
        // `Option<Lend<'short>>` when `'long: 'short`. This is only valid for
        // covariant types.
        const _: () = {
            #[allow(dead_code)]
            fn _check_covariance<'long: 'short, 'short>() {
                let x: Option<<$name as $crate::Lending<'long>>::Lend> = None;
                let _: Option<<$name as $crate::Lending<'short>>::Lend> = x;
            }
        };
    };
}

/// Implement the covariance check method for a [`FallibleLender`] impl with a
/// concrete [`Lend`](FallibleLending::Lend) type.
///
/// See [`check_covariance!`](crate::check_covariance!) for details.
#[macro_export]
macro_rules! check_covariance_fallible {
    () => {
        unsafe fn _check_covariance<'long: 'short, 'short>(
            lend: <Self as $crate::FallibleLending<'long>>::Lend,
        ) -> <Self as $crate::FallibleLending<'short>>::Lend {
            lend
        }
    };
}

/// Implement the covariance check method for adapter [`FallibleLender`] impls.
///
/// See [`inherit_covariance!`](crate::inherit_covariance!) for details.
#[macro_export]
macro_rules! inherit_covariance_fallible {
    () => {
        unsafe fn _check_covariance<'long: 'short, 'short>(
            lend: <Self as $crate::FallibleLending<'long>>::Lend,
        ) -> <Self as $crate::FallibleLending<'short>>::Lend {
            // SAFETY: Covariance is inherited from the underlying FallibleLender.
            unsafe { core::mem::transmute(lend) }
        }
    };
}

/// Internal struct used to implement [`fallible_lend!`], do not use directly.
#[doc(hidden)]
pub struct DynFallibleLendShunt<T: ?Sized>(pub T);

impl<'lend, T: ?Sized + for<'all> DynFallibleLend<'all>> FallibleLending<'lend> for DynFallibleLendShunt<T> {
    type Lend = <T as DynFallibleLend<'lend>>::Lend;
}

/// Internal trait used to implement [`lend!`], do not use directly.
#[doc(hidden)]
pub trait DynFallibleLend<'lend> {
    type Lend;
}

/// Use lifetime `'lend` within type `$T` to create an `impl for<'lend> FallibleLending<'lend, Lend = $T>`.
/// Uses a bug in the borrow checker which allows dyn objects to implement impossible traits.
/// # Examples
/// ```rust
/// use std::convert::Infallible;
///
/// use lender::prelude::*;
///
/// let mut empty = lender::fallible_empty::<Infallible, fallible_lend!(&'lend mut [u32])>();
/// let _: Result<Option<&mut [u32]>, Infallible> = empty.next(); // => Ok(None)
/// ```
#[macro_export]
macro_rules! fallible_lend {
    ($T:ty) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = $T>>
    };
}
