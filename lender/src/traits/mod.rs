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
    ext::{
        FallibleIteratorExt, FallibleIteratorRefExt, IntoFallibleIteratorExt, IntoIteratorExt,
        IteratorExt, IteratorRefExt,
    },
    fallible_lender::{FallibleLend, FallibleLender, FallibleLending},
    lender::{Lend, Lender, Lending},
    marker::{FusedFallibleLender, FusedLender},
};

/// Trait for lend types that can be destructured into two components, used
/// by [`Lender::unzip()`] and
/// [`FallibleLender::unzip()`](crate::FallibleLender::unzip).
///
/// Implemented for `(A, B)`, `&(A, B)`, and `&mut (A, B)`.
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

/// A covariance witness whose private field prevents construction
/// outside this crate, making it impossible to bypass covariance
/// checks without `unsafe`.
#[doc(hidden)]
pub struct CovariantProof<T>(core::marker::PhantomData<fn() -> T>);

impl<T> CovariantProof<T> {
    #[doc(hidden)]
    pub(crate) fn new() -> Self {
        CovariantProof(core::marker::PhantomData)
    }
}

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
///
/// The required method
/// [`__check_covariance`](CovariantLending::__check_covariance) enforces
/// covariance: its safe body `{ proof }` only compiles when the
/// [`Lend`](Lending::Lend) type is covariant, so non-covariant types cannot
/// implement this trait without `unsafe` with a returning body. See the
/// documentation of
/// [`Lender::__check_covariance`](crate::Lender::__check_covariance) for
/// details.
#[doc(hidden)]
pub trait CovariantLending: for<'all> Lending<'all> {
    fn __check_covariance<'long: 'short, 'short>(
        proof: CovariantProof<<Self as Lending<'long>>::Lend>,
    ) -> CovariantProof<<Self as Lending<'short>>::Lend>;
}

// SAFETY: lend!() only accepts type patterns that are syntactically covariant
// in 'lend (references, slices, tuples of identifiers, etc.)
impl<T: ?Sized + for<'all> DynLend<'all>> CovariantLending for DynLendShunt<T> {
    fn __check_covariance<'long: 'short, 'short>(
        proof: CovariantProof<<Self as Lending<'long>>::Lend>,
    ) -> CovariantProof<<Self as Lending<'short>>::Lend> {
        // SAFETY: lend!() only accepts syntactically covariant patterns
        unsafe { core::mem::transmute(proof) }
    }
}

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
    // Reference to str
    ($Shunt:ident, $Trait:ident, &$lt:lifetime str) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt str>>
    };
    // Mutable reference to str
    ($Shunt:ident, $Trait:ident, &$lt:lifetime mut str) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt mut str>>
    };
    // Reference to slice
    ($Shunt:ident, $Trait:ident, &$lt:lifetime [$T:ident]) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt [$T]>>
    };
    // Mutable reference to slice
    ($Shunt:ident, $Trait:ident, &$lt:lifetime mut [$T:ident]) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt mut [$T]>>
    };
    // Vec<T>
    ($Shunt:ident, $Trait:ident, Vec<$T:ident>) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = Vec<$T>>>
    };
    // Reference to Vec<T>
    ($Shunt:ident, $Trait:ident, &$lt:lifetime Vec<$T:ident>) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt Vec<$T>>>
    };
    // Mutable reference to Vec<T>
    ($Shunt:ident, $Trait:ident, &$lt:lifetime mut Vec<$T:ident>) => {
        $crate::$Shunt<dyn for<'lend> $crate::$Trait<'lend, Lend = &$lt mut Vec<$T>>>
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

/// Uses lifetime `'lend` within type `$T` to create an `impl for<'lend>
/// Lending<'lend, Lend = $T>`.
///
/// Uses a known borrow-checker bug which allows `dyn` objects to implement
/// impossible traits.
///
/// This macro only accepts type patterns that are guaranteed to be
/// covariant in `'lend`:
/// - Identifiers: `i32`, `String`, etc.
/// - References: `&'lend T`, `&'lend mut T`
/// - String slices: `&'lend str`, `&'lend mut str`
/// - Slices: `&'lend [T]`, `&'lend mut [T]`
/// - Vec: `Vec<T>`, `&'lend Vec<T>`, `&'lend mut Vec<T>`
/// - Double references: `& &'lend T`, `&&'lend T`
/// - Tuples of identifiers: `(T0,)`, `(T0, T1)`, `(T0, T1, T2)`, etc.
///   (any number of elements)
/// - References to tuples: `&'lend (T0,)`, `&'lend (T0, T1)`,
///   `&'lend mut (T0,)`, `&'lend mut (T0, T1)`, etc.
///
/// For types that are not covered by this macro, please use
/// [`covariant_lend!`](crate::covariant_lend!), which performs a compile-time
/// covariance check.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut empty = lender::empty::<lend!(&'lend mut [i32])>();
/// let _: Option<&mut [i32]> = empty.next(); // => None
/// ```
#[macro_export]
macro_rules! lend {
    // Identifier
    ($T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, $T) };
    // Reference patterns
    (&$lt:lifetime $T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt $T) };
    (&$lt:lifetime mut $T:ident) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt mut $T) };
    // String slices
    (&$lt:lifetime str) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt str) };
    (&$lt:lifetime mut str) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt mut str) };
    // Slices
    (&$lt:lifetime [$T:ident]) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt [$T]) };
    (&$lt:lifetime mut [$T:ident]) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt mut [$T]) };
    // Vec
    (Vec<$T:ident>) => { $crate::__lend_impl!(DynLendShunt, DynLend, Vec<$T>) };
    (&$lt:lifetime Vec<$T:ident>) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt Vec<$T>) };
    (&$lt:lifetime mut Vec<$T:ident>) => { $crate::__lend_impl!(DynLendShunt, DynLend, &$lt mut Vec<$T>) };
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
/// `__check_covariance` method implementation with body `{ proof }`, which
/// only compiles if the [`Lend`](Lending::Lend) type is covariant in its
/// lifetime.
///
/// For adapters that delegate to underlying lenders, use
/// [`unsafe_assume_covariance!`](crate::unsafe_assume_covariance!) instead.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
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
        fn __check_covariance<'long: 'short, 'short>(
            proof: $crate::CovariantProof<<Self as $crate::Lending<'long>>::Lend>,
        ) -> $crate::CovariantProof<<Self as $crate::Lending<'short>>::Lend> {
            proof
        }
    };
}

/// Skips the covariance check for [`Lender`] impls.
///
/// Use this macro for adapters whose [`Lend`](Lending::Lend) type is defined in
/// terms of another lender's [`Lend`](Lending::Lend) type (e.g., `type Lend =
/// Lend<'lend, L>`). The macro expands to the `__check_covariance` method
/// implementation with body `{ unsafe { core::mem::transmute(proof) } }`,
/// which skips the covariance check, as it always compiles.
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
        fn __check_covariance<'long: 'short, 'short>(
            proof: $crate::CovariantProof<<Self as $crate::Lending<'long>>::Lend>,
        ) -> $crate::CovariantProof<<Self as $crate::Lending<'short>>::Lend> {
            // SAFETY: Covariance is assumed by the caller of this macro
            unsafe { core::mem::transmute(proof) }
        }
    };
}

/// Defines a lending type with compile-time covariance checking.
///
/// This macro creates a struct that implements [`Lending`] and includes a
/// compile-time assertion that the lend type is covariant in its lifetime.
///
/// An optional visibility modifier can be provided to make the type reusable
/// across modules.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// // Define a covariance-checked lending type for &'lend Vec<i32>.
/// // This type cannot be expressed with lend!() because Vec<i32>
/// // has generics.
/// covariant_lend!(RefVec = &'lend Vec<i32>);
///
/// let data = [vec![1, 2], vec![3, 4]];
/// let mut lender = lender::lend_iter::<'_, RefVec, _>(data.iter());
/// let item: &Vec<i32> = lender.next().unwrap();
/// assert_eq!(item, &vec![1, 2]);
/// ```
///
/// With visibility modifier:
/// ```rust
/// # use lender::prelude::*;
/// // Define a public covariance-checked lending type.
/// covariant_lend!(pub MyLend = &'lend Vec<i32>);
///
/// // MyLend can be used outside the module.
/// let data = [vec![1, 2]];
/// let mut lender = lender::lend_iter::<'_, MyLend, _>(data.iter());
/// assert_eq!(lender.next(), Some(&vec![1, 2]));
/// ```
///
/// # Compile-time Error for Invariant Types
///
/// The following will fail to compile because
/// `Cell<Option<&'lend String>>` is invariant in `'lend`:
///
/// ```rust,compile_fail
/// # use lender::prelude::*;
/// # use std::cell::Cell;
/// // This fails to compile - Cell makes the type invariant!
/// covariant_lend!(
///     InvariantLend = &'lend Cell<Option<&'lend String>>
/// );
/// ```
#[macro_export]
macro_rules! covariant_lend {
    ($vis:vis $name:ident = $T:ty) => {
        /// A lending type with compile-time covariance checking.
        #[derive(Clone, Copy, Debug, Default)]
        $vis struct $name;

        impl<'lend> $crate::Lending<'lend> for $name {
            type Lend = $T;
        }

        // Covariance is enforced by the required _check_covariance method:
        // its body `{ proof }` only compiles if $T is covariant in 'lend.
        impl $crate::CovariantLending for $name {
            fn __check_covariance<'long: 'short, 'short>(
                proof: $crate::CovariantProof<<Self as $crate::Lending<'long>>::Lend>,
            ) -> $crate::CovariantProof<<Self as $crate::Lending<'short>>::Lend> {
                proof
            }
        }
    };
}

/// Defines a fallible lending type with compile-time covariance checking.
///
/// This macro creates a struct that implements [`FallibleLending`] and
/// includes a compile-time assertion that the lend type is covariant in its
/// lifetime.
///
/// An optional visibility modifier can be provided to make the type reusable
/// across modules.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// # use fallible_iterator::IteratorExt;
/// // Define a covariance-checked fallible lending type for
/// // Option<&'lend str>. This type cannot be expressed with
/// // fallible_lend!() because Option has generics.
/// covariant_fallible_lend!(OptStr = Option<&'lend str>);
///
/// let data = [Some("hello"), None, Some("world")];
/// let mut lender = lender::lend_fallible_iter::<'_, OptStr, _>(
///     data.iter().copied().into_fallible()
/// );
/// let item: Option<&str> = lender.next().unwrap().unwrap();
/// assert_eq!(item, Some("hello"));
/// ```
///
/// With visibility modifier:
/// ```rust
/// # use lender::prelude::*;
/// # use fallible_iterator::IteratorExt;
/// // Define a public covariance-checked fallible lending type.
/// covariant_fallible_lend!(pub MyLend = Option<&'lend str>);
///
/// // MyLend can be used outside the module.
/// let data = [Some("hello")];
/// let mut lender = lender::lend_fallible_iter::<'_, MyLend, _>(
///     data.iter().copied().into_fallible()
/// );
/// assert_eq!(lender.next().unwrap(), Some(Some("hello")));
/// ```
///
/// # Compile-time Error for Invariant Types
///
/// The following will fail to compile because
/// `Cell<Option<&'lend String>>` is invariant in `'lend`:
///
/// ```rust,compile_fail
/// # use lender::prelude::*;
/// # use std::cell::Cell;
/// // This fails to compile - Cell makes the type invariant!
/// covariant_fallible_lend!(
///     InvariantLend = &'lend Cell<Option<&'lend String>>
/// );
/// ```
#[macro_export]
macro_rules! covariant_fallible_lend {
    ($vis:vis $name:ident = $T:ty) => {
        /// A fallible lending type with compile-time covariance checking.
        #[derive(Clone, Copy, Debug, Default)]
        $vis struct $name;

        impl<'lend> $crate::FallibleLending<'lend> for $name {
            type Lend = $T;
        }

        // Covariance is enforced by the required _check_covariance method:
        // its body `{ proof }` only compiles if $T is covariant in 'lend.
        impl $crate::CovariantFallibleLending for $name {
            fn __check_covariance<'long: 'short, 'short>(
                proof: $crate::CovariantProof<&'short <Self as $crate::FallibleLending<'long>>::Lend>,
            ) -> $crate::CovariantProof<&'short <Self as $crate::FallibleLending<'short>>::Lend> {
                proof
            }
        }
    };
}

/// Implement the covariance check method for a [`FallibleLender`] impl with a
/// concrete [`Lend`](FallibleLending::Lend) type.
///
/// See [`check_covariance!`](crate::check_covariance!) for details.
#[macro_export]
macro_rules! check_covariance_fallible {
    () => {
        fn __check_covariance<'long: 'short, 'short>(
            proof: $crate::CovariantProof<&'short <Self as $crate::FallibleLending<'long>>::Lend>,
        ) -> $crate::CovariantProof<&'short <Self as $crate::FallibleLending<'short>>::Lend> {
            proof
        }
    };
}

/// Skips the covariance check for [`FallibleLender`] impls.
///
/// This is the fallible counterpart to [`unsafe_assume_covariance!`].
///
/// See [`unsafe_assume_covariance!`](crate::unsafe_assume_covariance!) for
/// more details.
#[macro_export]
macro_rules! unsafe_assume_covariance_fallible {
    () => {
        fn __check_covariance<'long: 'short, 'short>(
            proof: $crate::CovariantProof<&'short <Self as $crate::FallibleLending<'long>>::Lend>,
        ) -> $crate::CovariantProof<&'short <Self as $crate::FallibleLending<'short>>::Lend> {
            // SAFETY: Covariance is assumed by the caller of this macro
            unsafe { core::mem::transmute(proof) }
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
///
/// See [`CovariantLending`] for details on the covariance enforcement mechanism.
#[doc(hidden)]
pub trait CovariantFallibleLending: for<'all> FallibleLending<'all> {
    fn __check_covariance<'long: 'short, 'short>(
        proof: CovariantProof<&'short <Self as FallibleLending<'long>>::Lend>,
    ) -> CovariantProof<&'short <Self as FallibleLending<'short>>::Lend>;
}

// SAFETY: fallible_lend!() only accepts type patterns that are syntactically
// covariant in 'lend
impl<T: ?Sized + for<'all> DynFallibleLend<'all>> CovariantFallibleLending
    for DynFallibleLendShunt<T>
{
    fn __check_covariance<'long: 'short, 'short>(
        proof: CovariantProof<&'short <Self as FallibleLending<'long>>::Lend>,
    ) -> CovariantProof<&'short <Self as FallibleLending<'short>>::Lend> {
        // SAFETY: fallible_lend!() only accepts syntactically covariant patterns
        unsafe { core::mem::transmute(proof) }
    }
}

/// Internal trait used to implement [`fallible_lend!`], do not use directly.
#[doc(hidden)]
pub trait DynFallibleLend<'lend> {
    type Lend;
}

/// Uses lifetime `'lend` within type `$T` to create an `impl for<'lend>
/// FallibleLending<'lend, Lend = $T>`.
///
/// Uses a known borrow-checker bug which allows `dyn` objects to implement
/// impossible traits.
///
/// This macro only accepts type patterns that are guaranteed to be
/// covariant in `'lend`:
/// - Identifiers: `i32`, `String`, etc.
/// - References: `&'lend T`, `&'lend mut T`
/// - String slices: `&'lend str`, `&'lend mut str`
/// - Slices: `&'lend [T]`, `&'lend mut [T]`
/// - Vec: `Vec<T>`, `&'lend Vec<T>`, `&'lend mut Vec<T>`
/// - Double references: `& &'lend T`, `&&'lend T`
/// - Tuples of identifiers: `(T0,)`, `(T0, T1)`, `(T0, T1, T2)`, etc.
///   (any number of elements)
/// - References to tuples: `&'lend (T0,)`, `&'lend (T0, T1)`,
///   `&'lend mut (T0,)`, `&'lend mut (T0, T1)`, etc.
///
/// For types that are not covered by this macro, please use
/// [`covariant_fallible_lend!`](crate::covariant_fallible_lend!), which
/// performs a compile-time covariance check.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// # use std::convert::Infallible;
/// let mut empty = lender::fallible_empty::<
///     fallible_lend!(&'lend mut [i32]),
///     Infallible
/// >();
/// let _: Result<Option<&mut [i32]>, Infallible> = empty.next(); // => Ok(None)
/// ```
#[macro_export]
macro_rules! fallible_lend {
    // Identifier
    ($T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, $T) };
    // Reference patterns
    (&$lt:lifetime $T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt $T) };
    (&$lt:lifetime mut $T:ident) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt mut $T) };
    // String slices
    (&$lt:lifetime str) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt str) };
    (&$lt:lifetime mut str) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt mut str) };
    // Slices
    (&$lt:lifetime [$T:ident]) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt [$T]) };
    (&$lt:lifetime mut [$T:ident]) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt mut [$T]) };
    // Vec
    (Vec<$T:ident>) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, Vec<$T>) };
    (&$lt:lifetime Vec<$T:ident>) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt Vec<$T>) };
    (&$lt:lifetime mut Vec<$T:ident>) => { $crate::__lend_impl!(DynFallibleLendShunt, DynFallibleLend, &$lt mut Vec<$T>) };
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
