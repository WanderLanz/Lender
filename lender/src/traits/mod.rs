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
    collect::{
        ExtendFallibleLender, ExtendLender, FromFallibleLender, FromLender, IntoFallibleLender,
        IntoLender,
    },
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

/// Marker trait certifying that the [`Lend`](Lending::Lend) type is covariant
/// in its lifetime parameter.
///
/// This trait is automatically implemented for types created by [`lend!`] and
/// [`covariant_lend!`]. It is required by [`lend_iter`](crate::lend_iter) and
/// similar functions that perform lifetime transmutes which are only sound when
/// the lend type is covariant.
///
/// Users should not need to implement this trait directly. Use [`lend!`] for
/// simple patterns or [`covariant_lend!`] for complex types.
#[doc(hidden)]
pub trait CovariantLending: for<'all> Lending<'all> {}

// SAFETY: lend!() only accepts type patterns that are syntactically covariant
// in 'lend (references, slices, tuples of identifiers, etc.)
impl<T: ?Sized + for<'all> DynLend<'all>> CovariantLending for DynLendShunt<T> {}

/// Internal trait used to implement [`lend!`], do not use directly.
#[doc(hidden)]
pub trait DynLend<'lend> {
    type Lend;
}

/// Internal macro used by [`lend!`] and [`fallible_lend!`] to avoid duplication.
/// Do not use directly.
#[doc(hidden)]
#[macro_export]
macro_rules! __lend_impl {
    // Identifier (no lifetime - trivially covariant)
    ($Shunt:ident, $Trait:ident, $T:ident) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = $T>>
    };
    // Reference to identifier
    ($Shunt:ident, $Trait:ident, &$lt:lifetime $T:ident) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt $T>>
    };
    // Mutable reference to identifier
    ($Shunt:ident, $Trait:ident, &$lt:lifetime mut $T:ident) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt mut $T>>
    };
    // Reference to slice
    ($Shunt:ident, $Trait:ident, &$lt:lifetime [$T:ident]) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt [$T]>>
    };
    // Mutable reference to slice
    ($Shunt:ident, $Trait:ident, &$lt:lifetime mut [$T:ident]) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt mut [$T]>>
    };
    // Double reference (& &'lend T)
    ($Shunt:ident, $Trait:ident, & &$lt:lifetime $T:ident) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = & &$lt $T>>
    };
    // Double reference (&&'lend T - no space)
    ($Shunt:ident, $Trait:ident, &&$lt:lifetime $T:ident) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &&$lt $T>>
    };
    // Double mutable reference
    ($Shunt:ident, $Trait:ident, & &$lt:lifetime mut $T:ident) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = & &$lt mut $T>>
    };
    // Double mutable reference (no space)
    ($Shunt:ident, $Trait:ident, &&$lt:lifetime mut $T:ident) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &&$lt mut $T>>
    };
    // Reference to tuple (variadic)
    ($Shunt:ident, $Trait:ident, &$lt:lifetime ($($T:ident),+)) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt ($($T),+)>>
    };
    // Mutable reference to tuple (variadic)
    ($Shunt:ident, $Trait:ident, &$lt:lifetime mut ($($T:ident),+)) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt mut ($($T),+)>>
    };
    // Tuple (variadic)
    ($Shunt:ident, $Trait:ident, ($($T:ident),+)) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = ($($T),+)>>
    };
}

/// Use lifetime `'lend` within type `$T` to create an `impl for<'lend>
/// Lending<'lend, Lend = $T>`.
///
/// Uses a known borrow-checker bug which allows `dyn` objects to implement
/// impossible traits.
///
/// This macro only accepts type patterns that are guaranteed to be covariant in `'lend`:
/// - Identifiers: `u8`, `String`, etc.
/// - References: `&'lend T`, `&'lend mut T`
/// - Slices: `&'lend [T]`, `&'lend mut [T]`
/// - Double references: `& &'lend T`, `&&'lend T`
/// - Tuples of identifiers: `(T0,)`, `(T0, T1)`, `(T0, T1, T2)`, etc. (any number of elements)
/// - References to tuples: `&'lend (T0,)`, `&'lend (T0, T1)`, `&'lend mut (T0,)`, `&'lend mut (T0, T1)`, etc.
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
    // Identifier
    ($T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, $T) };
    // Reference patterns
    (&$lt:lifetime $T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt $T) };
    (&$lt:lifetime mut $T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt mut $T) };
    (&$lt:lifetime [$T:ident]) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt [$T]) };
    (&$lt:lifetime mut [$T:ident]) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt mut [$T]) };
    // Double references
    (& &$lt:lifetime $T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, & &$lt $T) };
    (&&$lt:lifetime $T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, &&$lt $T) };
    (& &$lt:lifetime mut $T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, & &$lt mut $T) };
    (&&$lt:lifetime mut $T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, &&$lt mut $T) };
    // References to tuples
    (&$lt:lifetime ($($T:ident),+ $(,)?)) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt ($($T),+)) };
    (&$lt:lifetime mut ($($T:ident),+ $(,)?)) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt mut ($($T),+)) };
    // Tuples
    (($($T:ident),+ $(,)?)) => { $crate::__lend_impl!(DynLendShunt, DynLend, ($($T),+)) };
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
/// [`unsafe_assume_covariance!`](crate::unsafe_assume_covariance!) instead.
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
        fn _check_covariance<'long: 'short, 'short>(
            lend: *const &'short <Self as Lending<'long>>::Lend,
            _: $crate::Uncallable,
        ) -> *const &'short <Self as Lending<'short>>::Lend {
            lend
        }
    };
}

/// Skips the covariance check for [`Lender`] impls.
///
/// Use this macro for adapters whose [`Lend`](Lending::Lend) type is defined in
/// terms of another lender's [`Lend`](Lending::Lend) type (e.g., `type Lend =
/// Lend<'lend, L>`). The macro expands to the `_check_covariance` method
/// implementation with body `{ unsafe { core::mem::transmute(lend) } }`, which
/// skips the covariance check, as it always compiles.
///
/// For lenders with concrete [`Lend`](Lending::Lend) types, use
/// [`check_covariance!`] instead.
///
/// # Examples
///
/// ```rust,ignore
/// impl<L: Lender> Lender for MyAdapter<L> {
///     // SAFETY: the lend is that of L
///     unsafe_assume_covariance!();  
///     // ...
/// }
/// ```
///
/// # Safety
///
/// This macro disables the covariance check using unsafe code. It is the
/// caller's responsibility to ensure that the adapter's [`Lend`](Lending::Lend)
/// type is indeed covariant in its lifetime. Please write a brief SAFETY
/// comment explaining why covariance holds, as you would for any other unsafe
/// code.
#[macro_export]
macro_rules! unsafe_assume_covariance {
    () => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        fn _check_covariance<'long: 'short, 'short>(
            lend: *const &'short <Self as Lending<'long>>::Lend,
            _: $crate::Uncallable,
        ) -> *const &'short <Self as Lending<'short>>::Lend {
            // SAFETY: Covariance is assumed by the caller of this macro
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

        // Covariance is certified by the const check below.
        impl $crate::CovariantLending for $name {}

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
/// lender::covariant_fallible_lend!(OptStr = Option<&'lend str>);
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
/// lender::covariant_fallible_lend!(InvariantLend = &'lend Cell<Option<&'lend String>>);
/// ```
#[macro_export]
macro_rules! covariant_fallible_lend {
    ($name:ident = $T:ty) => {
        /// A fallible lending type with compile-time covariance checking.
        #[derive(Clone, Copy, Debug, Default)]
        struct $name;

        impl<'lend> $crate::FallibleLending<'lend> for $name {
            type Lend = $T;
        }

        // Covariance is certified by the const check below.
        impl $crate::CovariantFallibleLending for $name {}

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
        fn _check_covariance<'long: 'short, 'short>(
            lend: *const &'short <Self as FallibleLending<'long>>::Lend,
            _: $crate::Uncallable,
        ) -> *const &'short <Self as FallibleLending<'short>>::Lend {
            lend
        }
    };
}

/// Skips the covariance check for [`FallibleLender`] impls.
///
/// This is the fallible counterpart to [`unsafe_assume_covariance!`].
///
/// See [`unsafe_assume_covariance!`](crate::unsafe_assume_covariance!) for more details.
#[macro_export]
macro_rules! unsafe_assume_covariance_fallible {
    () => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        fn _check_covariance<'long: 'short, 'short>(
            lend: *const &'short <Self as FallibleLending<'long>>::Lend,
            _: $crate::Uncallable,
        ) -> *const &'short <Self as FallibleLending<'short>>::Lend {
            // SAFETY: Covariance is assumed by the caller of this macro
            unsafe { core::mem::transmute(lend) }
        }
    };
}

/// Internal struct used to implement [`fallible_lend!`], do not use directly.
#[doc(hidden)]
pub struct DynFallibleLendShunt<T: ?Sized>(pub T);

impl<'lend, T: ?Sized + for<'all> DynFallibleLend<'all>> FallibleLending<'lend>
    for DynFallibleLendShunt<T>
{
    type Lend = <T as DynFallibleLend<'lend>>::Lend;
}

/// Fallible counterpart to [`CovariantLending`].
#[doc(hidden)]
pub trait CovariantFallibleLending: for<'all> FallibleLending<'all> {}

// SAFETY: fallible_lend!() only accepts type patterns that are syntactically
// covariant in 'lend
impl<T: ?Sized + for<'all> DynFallibleLend<'all>> CovariantFallibleLending
    for DynFallibleLendShunt<T>
{
}

/// Internal trait used to implement [`fallible_lend!`], do not use directly.
#[doc(hidden)]
pub trait DynFallibleLend<'lend> {
    type Lend;
}

/// Use lifetime `'lend` within type `$T` to create an `impl for<'lend>
/// FallibleLending<'lend, Lend = $T>`.
///
/// Uses a known borrow-checker bug which allows `dyn` objects to implement
/// impossible traits.
///
/// This macro only accepts type patterns that are guaranteed to be covariant in `'lend`:
/// - Identifiers: `u8`, `String`, etc.
/// - References: `&'lend T`, `&'lend mut T`
/// - Slices: `&'lend [T]`, `&'lend mut [T]`
/// - Double references: `& &'lend T`, `&&'lend T`
/// - Tuples of identifiers: `(T0,)`, `(T0, T1)`, `(T0, T1, T2)`, etc. (any number of elements)
/// - References to tuples: `&'lend (T0,)`, `&'lend (T0, T1)`, `&'lend mut (T0,)`, `&'lend mut (T0, T1)`, etc.
///
/// For types that are not covered by this macro, please use
/// [`covariant_fallible_lend!`](crate::covariant_fallible_lend!), which
/// performs a compile-time covariance check.
///
/// # Examples
/// ```rust
/// use std::convert::Infallible;
///
/// use lender::prelude::*;
///
/// let mut empty = lender::fallible_empty::<fallible_lend!(&'lend mut [u32]), Infallible>();
/// let _: Result<Option<&mut [u32]>, Infallible> = empty.next(); // => Ok(None)
/// ```
#[macro_export]
macro_rules! fallible_lend {
    // Identifier
    ($T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, $T) };
    // Reference patterns
    (&$lt:lifetime $T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt $T) };
    (&$lt:lifetime mut $T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt mut $T) };
    (&$lt:lifetime [$T:ident]) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt [$T]) };
    (&$lt:lifetime mut [$T:ident]) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt mut [$T]) };
    // Double references
    (& &$lt:lifetime $T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, & &$lt $T) };
    (&&$lt:lifetime $T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &&$lt $T) };
    (& &$lt:lifetime mut $T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, & &$lt mut $T) };
    (&&$lt:lifetime mut $T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &&$lt mut $T) };
    // References to tuples
    (&$lt:lifetime ($($T:ident),+ $(,)?)) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt ($($T),+)) };
    (&$lt:lifetime mut ($($T:ident),+ $(,)?)) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt mut ($($T),+)) };
    // Tuples
    (($($T:ident),+ $(,)?)) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, ($($T),+)) };
    // Fallback - reject with helpful error
    ($($tt:tt)*) => {
        compile_error!(concat!(
            "fallible_lend!() only accepts simple covariant patterns. ",
            "For complex types, use covariant_fallible_lend!(Name = YourType) instead."
        ))
    };
}
