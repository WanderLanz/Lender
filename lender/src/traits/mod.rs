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

/// Uninhabited type used to make `_check_covariance` methods uncallable.
#[doc(hidden)]
pub enum Uncallable {}

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

/// Use lifetime `'lend` within type `$T` to create an `impl for<'lend>
/// Lending<'lend, Lend = $T>`.
///
/// Uses a bug in the borrow checker which allows `dyn` objects to implement
/// impossible traits.
///
/// This macro only accepts type patterns that are guaranteed to be covariant in `'lend`:
/// - Identifiers: `u8`, `String`, etc.
/// - References: `&'lend T`, `&'lend mut T`
/// - Slices: `&'lend [T]`, `&'lend mut [T]`
/// - Double references: `& &'lend T`, `&&'lend T`
/// - Tuples of identifiers: `(T1, T2)`, `(T1, T2, T3)`, etc. (up to 10 elements)
/// - References to tuples: `&'lend (T1, T2)`, `&'lend mut (T1, T2)`, etc.
///
/// For types that are not covered by this macro, please use
/// [`covariant_lend!`](crate::covariant_lend!), which performs a compile-time
/// covariance check.
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut empty = lender::empty::<lend!(&'lend mut [u32])>();
/// let _: Option<&mut [u32]> = empty.next(); // => None
/// ```
#[macro_export]
macro_rules! lend {
    // Identifier (no lifetime - trivially covariant)
    ($T:ident) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = $T>>
    };
    // Reference to identifier
    (&$lt:lifetime $T:ident) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt $T>>
    };
    // Mutable reference to identifier
    (&$lt:lifetime mut $T:ident) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut $T>>
    };
    // Reference to slice
    (&$lt:lifetime [$T:ident]) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt [$T]>>
    };
    // Mutable reference to slice
    (&$lt:lifetime mut [$T:ident]) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut [$T]>>
    };
    // Double reference (& &'lend T)
    (& &$lt:lifetime $T:ident) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = & &$lt $T>>
    };
    // Double reference (&&'lend T - no space)
    (&&$lt:lifetime $T:ident) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &&$lt $T>>
    };
    // Double mutable reference
    (& &$lt:lifetime mut $T:ident) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = & &$lt mut $T>>
    };
    // Double mutable reference (no space)
    (&&$lt:lifetime mut $T:ident) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &&$lt mut $T>>
    };

    // References to tuples of identifiers (1 to 10 elements)
    (&$lt:lifetime ($T1:ident,)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1,)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1, $T2)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1, $T2, $T3)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6, $T7)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident, $T10:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10)>>
    };

    // Mutable references to tuples of identifiers (1 to 10 elements)
    (&$lt:lifetime mut ($T1:ident,)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1,)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1, $T2)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1, $T2, $T3)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6, $T7)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident, $T10:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10)>>
    };

    // Tuples of identifiers (1 to 10 elements)
    (($T1:ident,)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1,)>>
    };
    (($T1:ident, $T2:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1, $T2)>>
    };
    (($T1:ident, $T2:ident, $T3:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1, $T2, $T3)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1, $T2, $T3, $T4)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6, $T7)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident, $T10:ident)) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10)>>
    };

    // Fallback - reject with helpful error
    ($($tt:tt)*) => {
        compile_error!(concat!(
            "lend!() only accepts simple covariant patterns. ",
            "For complex types, use covariant_lend!(Name = YourType) instead."
        ))
    };
}

/// Implement the covariance check method for a [`Lender`] impl with a concrete
/// [`Lend`](Lending::Lend) type.
///
/// This macro must be invoked inside `impl Lender for T` blocks where the
/// [`Lend`](Lending::Lend) type is concrete (not defined in terms of another
/// lender's [`Lend`](Lending::Lend) type). It expands to the
/// `_check_covariance` method implementation with body `{ lend }`, which only
/// compiles if the [`Lend`](Lending::Lend) type is covariant in its lifetime.
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
#[macro_export]
macro_rules! check_covariance {
    () => {
        unsafe fn _check_covariance<'long: 'short, 'short>(
            lend: *const &'short <Self as Lending<'long>>::Lend,
            _: $crate::Uncallable,
        ) -> *const &'short <Self as Lending<'short>>::Lend {
            lend
        }
    };
}

/// Skips the covariance check for adapter [`Lender`] impls.
///
/// Use this macro for adapters whose [`Lend`](Lending::Lend) type is defined in
/// terms of another lender's [`Lend`](Lending::Lend) type (e.g., `type Lend =
/// Lend<'lend, L>`). The macro takes one or more underlying lending types as
/// parameters and verifies at compile time that each implements [`Lending`] for
/// all lifetimes, ensuring their [`Lend`](Lending::Lend) types have been
/// verified for covariance. It then expands to the `_check_covariance` method
/// implementation with body `{ unsafe { core::mem::transmute(lend) } }`, which
/// skips the covariance check, as it always compiles.
///
/// For lenders with concrete [`Lend`](Lending::Lend) types, use
/// [`check_covariance!`] instead.
///
/// Note that the purpose of specifying the underlying lending types is to avoid
/// the obvious footgun of using this macro with a concrete type, which would
/// make the covariance check vacuously true and lead to undefined behavior.
/// Nonetheless, it is still possible to misuse this macro.
///
/// In some cases this macro might not work—for example, when the underlying
/// lending type is fallible. In these cases, it is necessary to implement
/// the [`_check_covariance`](Lender::_check_covariance) method manually.
///
/// # Examples
///
/// ```rust,ignore
/// impl<L: Lender> Lender for MyAdapter<L> {
///     inherit_covariance!(L);  // Inherits covariance from L
///     // ...
/// }
///
/// impl<A: Lender, B: Lender> Lender for MyZipAdapter<A, B> {
///     inherit_covariance!(A, B);  // Inherits covariance from A and B
///     // ...
/// }
/// ```
///
/// # Safety
///
/// This macro uses transmute internally. It is safe because the
/// [`Lend`](Lending::Lend) type of the underlying [`Lending`] types is covariant.
/// Since this adapter's [`Lend`](Lending::Lend) type is derived from the
/// underlying types', the covariance is transitively guaranteed.
#[macro_export]
macro_rules! inherit_covariance {
    ($($L:ty),+ $(,)?) => {
        unsafe fn _check_covariance<'long: 'short, 'short>(
            lend: *const &'short <Self as Lending<'long>>::Lend,
            _: $crate::Uncallable,
        ) -> *const &'short <Self as Lending<'short>>::Lend {
            // Compile-time check: each type must implement Lending for all lifetimes,
            // ensuring its Lend type has already been verified for covariance.
            fn _assert_lending<L: ?Sized + for<'all> $crate::Lending<'all>>() {}
            $(_assert_lending::<$L>();)+

            // SAFETY: Covariance is inherited from the underlying Lending types
            unsafe { core::mem::transmute(lend) }
        }
    };
}

/// Defines a lending type with compile-time covariance checking.
///
/// This macro creates a struct that implements [`Lending`] and includes a compile-time
/// assertion that the lend type is covariant in its lifetime.
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
///
/// // Define a covariance-checked lending type for &'lend Vec<u8>.
/// // This type cannot be expressed with lend!() because Vec<u8> has generics.
/// lender::covariant_lend!(RefVec = &'lend Vec<u8>);
///
/// let data = [vec![1u8, 2], vec![3, 4]];
/// let mut lender = lender::lend_iter::<'_, RefVec, _>(data.iter());
/// let item: &Vec<u8> = lender.next().unwrap();
/// assert_eq!(item, &vec![1u8, 2]);
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
        // The check uses pointer coercion: *const &'short Lend<'long> can only
        // coerce to *const &'short Lend<'short> if Lend is covariant.
        const _: () = {
            #[allow(dead_code)]
            fn _check_covariance<'long: 'short, 'short>(
                lend: *const &'short <$name as $crate::Lending<'long>>::Lend,
            ) -> *const &'short <$name as $crate::Lending<'short>>::Lend {
                lend
            }
        };
    };
}

/// Defines a fallible lending type with compile-time covariance checking.
///
/// This macro creates a struct that implements [`FallibleLending`] and includes a
/// compile-time assertion that the lend type is covariant in its lifetime.
///
/// # Examples
/// ```rust
/// use fallible_iterator::IteratorExt as _;
/// use lender::prelude::*;
///
/// // Define a covariance-checked fallible lending type for Option<&'lend str>.
/// // This type cannot be expressed with fallible_lend!() because Option has generics.
/// lender::covariant_lend_fallible!(OptStr = Option<&'lend str>);
///
/// let data = [Some("hello"), None, Some("world")];
/// let mut lender = lender::lend_fallible_iter::<'_, OptStr, _>(data.iter().copied().into_fallible());
/// let item: Option<&str> = lender.next().unwrap().unwrap();
/// assert_eq!(item, Some("hello"));
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
/// lender::covariant_lend_fallible!(InvariantLend = &'lend Cell<Option<&'lend String>>);
/// ```
#[macro_export]
macro_rules! covariant_lend_fallible {
    ($name:ident = $T:ty) => {
        /// A fallible lending type with compile-time covariance checking.
        #[derive(Clone, Copy, Debug, Default)]
        struct $name;

        impl<'lend> $crate::FallibleLending<'lend> for $name {
            type Lend = $T;
        }

        // Covariance check: this const only compiles if $T is covariant in 'lend.
        // The check uses pointer coercion: *const &'short Lend<'long> can only
        // coerce to *const &'short Lend<'short> if Lend is covariant.
        const _: () = {
            #[allow(dead_code)]
            fn _check_covariance<'long: 'short, 'short>(
                lend: *const &'short <$name as $crate::FallibleLending<'long>>::Lend,
            ) -> *const &'short <$name as $crate::FallibleLending<'short>>::Lend {
                lend
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
            lend: *const &'short <Self as FallibleLending<'long>>::Lend,
            _: $crate::Uncallable,
        ) -> *const &'short <Self as FallibleLending<'short>>::Lend {
            lend
        }
    };
}

/// Skips the covariance check for adapter [`FallibleLender`] impls.
///
/// This is the fallible counterpart to [`inherit_covariance!`]. It takes one or
/// more underlying fallible lending types as parameters and verifies at compile
/// time that each implements [`FallibleLending`] for all lifetimes.
///
/// See [`inherit_covariance!`](crate::inherit_covariance!) for more details.
#[macro_export]
macro_rules! inherit_covariance_fallible {
    ($($L:ty),+ $(,)?) => {
        unsafe fn _check_covariance<'long: 'short, 'short>(
            lend: *const &'short <Self as FallibleLending<'long>>::Lend,
            _: $crate::Uncallable,
        ) -> *const &'short <Self as FallibleLending<'short>>::Lend {
            // Compile-time check: each type must implement FallibleLending for all lifetimes,
            // ensuring its Lend type has already been verified for covariance.
            fn _assert_fallible_lending<L: ?Sized + for<'all> $crate::FallibleLending<'all>>() {}
            $(_assert_fallible_lending::<$L>();)+

            // SAFETY: Covariance is inherited from the underlying FallibleLending types
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

/// Use lifetime `'lend` within type `$T` to create an `impl for<'lend>
/// FallibleLending<'lend, Lend = $T>`.
///
/// Uses a bug in the borrow checker which allows `dyn` objects to implement
/// impossible traits.
///
/// This macro only accepts type patterns that are guaranteed to be covariant in `'lend`:
/// - Identifiers: `u8`, `String`, etc.
/// - References: `&'lend T`, `&'lend mut T`
/// - Slices: `&'lend [T]`, `&'lend mut [T]`
/// - Double references: `& &'lend T`, `&&'lend T`
/// - Tuples of identifiers: `(T1, T2)`, `(T1, T2, T3)`, etc. (up to 10 elements)
/// - References to tuples: `&'lend (T1, T2)`, `&'lend mut (T1, T2)`, etc.
///
/// For types that are not covered by this macro, please use
/// [`covariant_lend_fallible!`](crate::covariant_lend_fallible!), which
/// performs a compile-time covariance check.
///
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
    // Identifier (no lifetime - trivially covariant)
    ($T:ident) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = $T>>
    };
    // Reference to identifier
    (&$lt:lifetime $T:ident) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt $T>>
    };
    // Mutable reference to identifier
    (&$lt:lifetime mut $T:ident) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut $T>>
    };
    // Reference to slice
    (&$lt:lifetime [$T:ident]) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt [$T]>>
    };
    // Mutable reference to slice
    (&$lt:lifetime mut [$T:ident]) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut [$T]>>
    };
    // Double reference (& &'lend T)
    (& &$lt:lifetime $T:ident) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = & &$lt $T>>
    };
    // Double reference (&&'lend T - no space)
    (&&$lt:lifetime $T:ident) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &&$lt $T>>
    };
    // Double mutable reference
    (& &$lt:lifetime mut $T:ident) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = & &$lt mut $T>>
    };
    // Double mutable reference (no space)
    (&&$lt:lifetime mut $T:ident) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &&$lt mut $T>>
    };

    // References to tuples of identifiers (1 to 10 elements)
    (&$lt:lifetime ($T1:ident,)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1,)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1, $T2)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1, $T2, $T3)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6, $T7)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9)>>
    };
    (&$lt:lifetime ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident, $T10:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10)>>
    };

    // Mutable references to tuples of identifiers (1 to 10 elements)
    (&$lt:lifetime mut ($T1:ident,)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1,)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1, $T2)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1, $T2, $T3)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6, $T7)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9)>>
    };
    (&$lt:lifetime mut ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident, $T10:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = &$lt mut ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10)>>
    };

    // Tuples of identifiers (1 to 10 elements)
    (($T1:ident,)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1,)>>
    };
    (($T1:ident, $T2:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1, $T2)>>
    };
    (($T1:ident, $T2:ident, $T3:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1, $T2, $T3)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1, $T2, $T3, $T4)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6, $T7)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9)>>
    };
    (($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident, $T10:ident)) => {
        $crate::DynFallibleLendShunt<dyn for<'lend> $crate::DynFallibleLend<'lend, Lend = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10)>>
    };

    // Fallback - reject with helpful error
    ($($tt:tt)*) => {
        compile_error!(concat!(
            "fallible_lend!() only accepts simple covariant patterns. ",
            "For complex types, use covariant_lend!(Name = YourType) instead."
        ))
    };
}
