//! # Types and Traits for Higher-Order Types with Covariance Checks
//!
//! - Flexible function signatures, to work around function lifetime
//!   signature restrictions.
//!
//! - Higher-Rank Closures, with macro checking covariance of return
//!   types with respect to bound lifetimes.
//!
//! - [`Covar`] transparent wrapper to mark covariance-checked
//!   closures.
//!
//! Use the [`covar!`](`crate::covar`),
//! [`covar_mut!`](`crate::covar_mut`), or
//! [`covar_once!`](`crate::covar_once`) macros to create a
//! covariance-checked higher-rank closure wrapped in [`Covar`].
//!
//! These macros are modified versions of the
//! [`higher_order_closure!`](https://crates.io/crates/higher-order-closure)
//! macro, adding a covariance check and the [`Covar`] wrapper.
//! They share the same underlying mechanism: a "funnel" helper
//! function whose `Fn` trait bounds enforce the desired higher-order
//! signature on the closure.
//!
//! # Lifetime elision
//!
//! When the closure has a return type, an explicit `for<'a>` clause
//! is **required** so that the macro can name the lifetime it
//! checks for covariance:
//!
//! ```rust
//! # use lender::prelude::*;
//! let mut data = [1, 2, 3];
//! let mut lender = lender::windows_mut(&mut data, 2)
//!     .map(covar_mut!(
//!         for<'lend> |w: &'lend mut [i32]| -> &'lend mut i32 {
//!             &mut w[0]
//!         }
//!     ));
//! assert_eq!(lender.next(), Some(&mut 1));
//! ```
//!
//! When the closure has **no** return type (i.e., it returns `()`),
//! the `for<>` clause can be omitted. In that case, [lifetime
//! elision rules for function
//! signatures](https://doc.rust-lang.org/reference/lifetime-elision.html#lifetime-elision-in-functions)
//! apply inside the funnel's `Fn` trait bound, making the
//! closure higher-order automatically. No covariance check is
//! needed because `()` is trivially covariant.
//!
//! ```rust
//! # use lender::prelude::*;
//! let mut data = [0, 1, 0, 0, 0, 0, 0, 0, 0];
//! lender::windows_mut(&mut data, 3).for_each(
//!     covar_mut!(|w: &mut [i32]| {
//!         w[2] = w[0] + w[1];
//!     }).into_inner()
//! );
//! assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
//! ```
//!
//! # Outer generic parameters: `#![with<…>]`
//!
//! The macros internally generate a helper function, so generic
//! parameters from the enclosing scope are not directly available
//! in the closure signature (similar to how nested `fn` items
//! cannot use outer generics). The `#![with<…>]` attribute
//! re-introduces them:
//!
//! ```rust
//! # use lender::prelude::*;
//! fn display_first<T: core::fmt::Display>(data: &mut [T]) {
//!     let mut lender = lender::windows_mut(data, 1)
//!         .map(covar_mut!(
//!             #![with<T: core::fmt::Display>]
//!             for<'lend> |w: &'lend mut [T]|
//!                 -> &'lend dyn core::fmt::Display
//!             {
//!                 &w[0]
//!             }
//!         ));
//!     if let Some(d) = lender.next() {
//!         println!("{d}");
//!     }
//! }
//! # display_first(&mut [1, 2, 3]);
//! ```
//!
//! The generics inside `#![with<…>]` support a restricted syntax:
//!
//! - **Lifetime parameters** with at most one super-lifetime bound:
//!   `'a`, `'b: 'a`.
//!
//! - **Type parameters** with an optional `?Sized`, followed by an
//!   optional lifetime bound, followed by an optional trait bound:
//!   `T`, `U: ?Sized + 'a + core::fmt::Debug`.
//!
//! For bounds that exceed this "simple shape", use a `where` clause
//! after the generics:
//!
//! ```rust
//! # use lender::prelude::*;
//! fn display_debug<T>(data: &mut [T])
//! where
//!     T: core::fmt::Display + core::fmt::Debug,
//! {
//!     let mut lender = lender::windows_mut(data, 1)
//!         .map(covar_mut!(
//!             #![with<T>
//!                 where T: core::fmt::Display
//!                        + core::fmt::Debug
//!             ]
//!             for<'lend> |w: &'lend mut [T]|
//!                 -> &'lend dyn core::fmt::Display
//!             {
//!                 &w[0]
//!             }
//!         ));
//!     if let Some(d) = lender.next() {
//!         println!("{d}");
//!     }
//! }
//! # display_debug(&mut [1, 2, 3]);
//! ```
//!
//! In practice, bounds inside `#![with<…>]` are seldom needed
//! because the generics are only used for the *signature* of the
//! closure, not its body.

/// A transparent wrapper that seals a closure whose covariance has been
/// verified at construction time by the [`covar!`](crate::covar),
/// [`covar_mut!`](crate::covar_mut), or [`covar_once!`](crate::covar_once)
/// macros.
///
/// Adapter structs like [`Map`](crate::Map) and sources like
/// [`once_with`](crate::once_with) store `Covar<F>` and call the inner closure
/// through [`as_inner`](Covar::as_inner) and
/// [`as_inner_mut`](Covar::as_inner_mut), or [`into_inner`](Covar::into_inner).
///
/// `Covar<F>` cannot be constructed safely by user code (the
/// constructor is unsafe, and also hidden to discourage usage).
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Covar<F>(F);

impl<F> Covar<F> {
    /// Creates a new `Covar<F>`.
    ///
    /// Please don't use this constructor unless you really need to. The main
    /// usage is creation of covariance-checked closures in the
    /// [`covar!`](crate::covar), [`covar_mut!`](crate::covar_mut), and
    /// [`covar_once!`](crate::covar_once) macros.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `f` produces covariant output with
    /// respect to any lifetime parameters in its signature. This is
    /// guaranteed by the [`covar!`](crate::covar),
    /// [`covar_mut!`](crate::covar_mut), and
    /// [`covar_once!`](crate::covar_once) macros.
    #[doc(hidden)]
    #[inline(always)]
    pub unsafe fn __new(f: F) -> Self {
        Covar(f)
    }

    /// Returns a reference to the inner closure.
    #[inline(always)]
    pub fn as_inner(&self) -> &F {
        &self.0
    }

    /// Returns a mutable reference to the inner closure.
    #[inline(always)]
    pub fn as_inner_mut(&mut self) -> &mut F {
        &mut self.0
    }

    /// Unwraps and returns the inner closure.
    #[inline(always)]
    pub fn into_inner(self) -> F {
        self.0
    }
}

/// Higher-Kinded Associated Output [`FnOnce`], where `Output`
/// (`B`) has lifetime `'b`.
pub trait FnOnceHKA<'b, A>: FnOnce(A) -> <Self as FnOnceHKA<'b, A>>::B {
    type B: 'b;
}

impl<'b, A, B: 'b, F: FnOnce(A) -> B> FnOnceHKA<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnMut`], where `Output`
/// (`B`) has lifetime `'b`.
pub trait FnMutHKA<'b, A>: FnMut(A) -> <Self as FnMutHKA<'b, A>>::B {
    type B: 'b;
}

impl<'b, A, B: 'b, F: FnMut(A) -> B> FnMutHKA<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnMut`], where `Output`
/// ([`Option<B>`](Option)) has lifetime `'b`.
pub trait FnMutHKAOpt<'b, A>: FnMut(A) -> Option<<Self as FnMutHKAOpt<'b, A>>::B> {
    type B: 'b;
}

impl<'b, A, B: 'b, F: FnMut(A) -> Option<B>> FnMutHKAOpt<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnOnce`], where `Output`
/// ([`Result<B, E>`](Result)) has output type `B` with lifetime `'b`.
pub trait FnOnceHKARes<'b, A, E>:
    FnOnce(A) -> Result<<Self as FnOnceHKARes<'b, A, E>>::B, E>
{
    type B: 'b;
}

impl<'b, A, B: 'b, E, F: FnOnce(A) -> Result<B, E>> FnOnceHKARes<'b, A, E> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnMut`], where `Output`
/// ([`Result<B, E>`](Result)) has output type `B` with lifetime `'b`.
pub trait FnMutHKARes<'b, A, E>: FnMut(A) -> Result<<Self as FnMutHKARes<'b, A, E>>::B, E> {
    type B: 'b;
}

impl<'b, A, B: 'b, E, F: FnMut(A) -> Result<B, E>> FnMutHKARes<'b, A, E> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnMut`], where `Output`
/// (`Result<Option<B>, E>`) has output type `B` with lifetime `'b`.
pub trait FnMutHKAResOpt<'b, A, E>:
    FnMut(A) -> Result<Option<<Self as FnMutHKAResOpt<'b, A, E>>::B>, E>
{
    type B: 'b;
}

impl<'b, A, B: 'b, E, F: FnMut(A) -> Result<Option<B>, E>> FnMutHKAResOpt<'b, A, E> for F {
    type B = B;
}

/// Not meant to be called directly. A modified version of
/// [`higher-order-closure`](https://crates.io/crates/higher-order-closure)
/// `higher_order_closure` macro to use any [`Fn`] trait.
///
/// Performs a covariance check when a return type is specified with a
/// `for<>` bound.
#[doc(hidden)]
#[macro_export]
macro_rules! __covar__ {
    // Case 1: With for<'lifetime> and return type - includes covariance check
    //
    // $hr ("higher-rank") is the universally quantified lifetime
    // from the `for<'hr>` clause.  It is the lifetime whose
    // covariance in $Ret the macro checks.
    //
    // $lt ("lifetime") are optional *outer* lifetime parameters
    // introduced via `#![with<'a, 'b, …>]`.  They let $Ret
    // reference lifetimes from the enclosing scope that are NOT
    // higher-rank.  They appear in __CovarCheck and
    // __check_covariance but are passed through unchanged — only
    // $hr is tested for covariance.
    (
        $F:ident,
        $(#![
            with<
                $($(
                    $lt:lifetime $(: $super_lt:lifetime)?
                ),+ $(,)?)?
                $($(
                    $T:ident $(:
                        $(
                            ?$Sized:ident $(+)?
                        )?
                        $(
                            $super:lifetime $(+)?
                        )?
                        $(
                            $Trait:path
                        )?
                    )?
                ),+ $(,)?)?
            >
            $(where
                $($wc:tt)*
            )?
        ])?

        for<$hr:lifetime>
        $( move $(@$move:tt)?)?
        | $($arg:tt : $Arg:ty),* $(,)?|
        -> $Ret:ty
        $body:block
    ) => (
        // SAFETY: the covariance check inside __funnel__ guarantees
        // that the return type is covariant in the bound lifetime.
        unsafe { $crate::higher_order::Covar::__new(
        ({
            fn __funnel__<
                $(
                    $($(
                        $lt $(: $super_lt)?
                        ,
                    )+)?
                    $($(
                        $T
                        $(:
                            $(?$Sized +)?
                            $($super +)?
                            $($Trait)?
                        )?
                        ,
                    )+)?
                )?
                    __Closure,
                >
            (
                f: __Closure,
            ) -> __Closure
            where
                __Closure : for<$hr> ::core::ops::$F($($Arg),*) -> $Ret,
                $($($($wc)*)?)?
            {
                // Covariance check: this struct has the same variance as $Ret
                // with respect to the bound lifetime. The PhantomData<&'a ()>
                // ensures the lifetime is used even if $Ret doesn't contain it.
                #[allow(dead_code)]
                struct __CovarCheck<
                    $(
                        $($(
                            $lt,
                        )+)?
                    )?
                    $hr
                >(
                    ::core::marker::PhantomData<
                        (
                            fn() -> ($Ret, &$hr ()),
                            $(
                                $($(
                                    &$lt (),
                                )+)?
                            )?
                        )
                    >
                );

                // This function only compiles if __CovarCheck (and thus $Ret)
                // is covariant in the lifetime parameter. See the documentation of
                // Lender::__check_covariance for details.
                #[inline(always)]
                #[allow(dead_code)]
                fn __check_covariance<
                    $(
                        $($(
                            $lt,
                        )+)?
                    )?
                    '__long: '__short, '__short
                >(
                    x: *const __CovarCheck<
                        $(
                            $($(
                                $lt,
                            )+)?
                        )?
                        '__long
                    >,
                ) -> *const __CovarCheck<
                    $(
                        $($(
                            $lt,
                        )+)?
                    )?
                    '__short
                > {
                    x
                }

                f
            }

            __funnel__::<$($($($T ,)+)?)? _>
        })(
            $(move $($move)?)? |$($arg),*| $body
        )
        ) }
    );

    // Case 2: No return type - no covariance check needed
    //
    // Without a return type the closure returns `()`, which is
    // trivially covariant, so no check is required.
    //
    // A return type WITHOUT a `for<>` clause intentionally fails to
    // match either case, producing a compile error: the user must
    // provide an explicit `for<'a>` so that the covariance check in
    // Case 1 can name the lifetime it checks.
    (
        $F:ident,
        $(#![
            with<
                $($(
                    $lt:lifetime $(: $super_lt:lifetime)?
                ),+ $(,)?)?
                $($(
                    $T:ident $(:
                        $(
                            ?$Sized:ident $(+)?
                        )?
                        $(
                            $super:lifetime $(+)?
                        )?
                        $(
                            $Trait:path
                        )?
                    )?
                ),+ $(,)?)?
            >
            $(where
                $($wc:tt)*
            )?
        ])?

        $( for<$hr:lifetime> )?
        $( move $(@$move:tt)?)?
        | $($arg:tt : $Arg:ty),* $(,)?|
        $body:block
    ) => (
        // SAFETY: no return type means the closure returns `()`,
        // which is covariant in every lifetime.
        unsafe { $crate::higher_order::Covar::__new(
        ({
            fn __funnel__<
                $(
                    $($(
                        $lt $(: $super_lt)?
                        ,
                    )+)?
                    $($(
                        $T
                        $(:
                            $(?$Sized +)?
                            $($super +)?
                            $($Trait)?
                        )?
                        ,
                    )+)?
                )?
                    __Closure,
                >
            (
                f: __Closure,
            ) -> __Closure
            where
                __Closure : $(for<$hr>)? ::core::ops::$F($($Arg),*),
                $($($($wc)*)?)?
            {
                f
            }

            __funnel__::<$($($($T ,)+)?)? _>
        })(
            $(move $($move)?)? |$($arg),*| $body
        )
        ) }
    );
}

/// Covariance-checked [`FnOnce`] closure macro.
///
/// Creates a [`Covar`]-wrapped closure with explicit lifetime bounds and a
/// compile-time covariance check (when a return type is specified with
/// `for<'a>` bounds). This ensures that the return type is covariant in the
/// bound lifetime, which is required for soundness with lending iterators.
///
/// Use `covar_once!` when the closure will only be called once (e.g., with
/// [`Lender::once_with`](crate::once_with)). For closures that may be
/// called multiple times, use [`covar_mut!`](crate::covar_mut) or
/// [`covar!`](crate::covar).
///
/// # Syntax
///
/// ```text
/// covar_once!(for<'a> |arg: Type<'a>| -> ReturnType<'a> { body })
/// covar_once!(for<'a> move |arg: Type<'a>| -> ReturnType<'a> { body })
/// ```
///
/// # Examples
///
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::once_with(42,
///     covar_once!(for<'lend> |state: &'lend mut i32| -> &'lend mut i32 {
///         *state += 1;
///         state
///     })
/// );
/// assert_eq!(lender.next(), Some(&mut 43));
/// assert_eq!(lender.next(), None);
/// ```
#[macro_export]
macro_rules! covar_once {($($t:tt)+) => ($crate::__covar__!(FnOnce, $($t)+))}

/// Covariance-checked [`FnMut`] closure macro.
///
/// Creates a [`Covar`]-wrapped closure with explicit lifetime bounds and a
/// compile-time covariance check (when a return type is specified with
/// `for<'a>` bounds). This ensures that the return type is covariant in the
/// bound lifetime, which is required for soundness with lending iterators.
///
/// Use `covar_mut!` when the closure may be called multiple times and
/// captures mutable state or needs `&mut self` semantics. This is the most
/// commonly used variant for lender methods like
/// [`Lender::map`](crate::Lender::map),
/// [`Lender::for_each`](crate::Lender::for_each),
/// [`Lender::filter_map`](crate::Lender::filter_map), and
/// [`Lender::scan`](crate::Lender::scan).
///
/// # Syntax
///
/// ```text
/// covar_mut!(for<'a> |arg: Type<'a>| -> ReturnType<'a> { body })
/// covar_mut!(for<'a> move |arg: Type<'a>| -> ReturnType<'a> { body })
/// ```
///
/// # Examples
///
/// ```rust
/// # use lender::prelude::*;
/// let mut data = [1, 2, 3, 4];
/// let mut lender = lender::windows_mut(&mut data, 2)
///     .map(covar_mut!(for<'lend> |w: &'lend mut [i32]| -> &'lend mut i32 {
///         &mut w[0]
///     }));
/// assert_eq!(lender.next(), Some(&mut 1));
/// ```
///
/// ```rust
/// # use lender::prelude::*;
/// let mut data = [0, 1, 0, 0, 0, 0, 0, 0, 0];
/// lender::windows_mut(&mut data, 3)
///     .for_each(|w| {
///         w[2] = w[0] + w[1];  // Compute Fibonacci
///     });
/// assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
/// ```
#[macro_export]
macro_rules! covar_mut {($($t:tt)+) => ($crate::__covar__!(FnMut, $($t)+))}

/// Covariance-checked [`Fn`] closure macro.
///
/// Creates a [`Covar`]-wrapped closure with explicit lifetime bounds and a
/// compile-time covariance check (when a return type is specified with
/// `for<'a>` bounds). This ensures that the return type is covariant in the
/// bound lifetime, which is required for soundness with lending iterators.
///
/// Use `covar!` when the closure only needs shared access to its captures
/// (`&self` semantics) and may be called multiple times. In practice,
/// [`covar_mut!`](crate::covar_mut) is more commonly used since most lender
/// methods require [`FnMut`].
///
/// # Syntax
///
/// ```text
/// covar!(for<'a> |arg: Type<'a>| -> ReturnType<'a> { body })
/// covar!(for<'a> move |arg: Type<'a>| -> ReturnType<'a> { body })
/// ```
///
/// # Examples
///
/// ```rust
/// # use lender::prelude::*;
/// let data = [1, 2, 3];
/// let lender = data.iter().into_lender();
/// let mapped = lender.map(
///     covar!(for<'lend> |x: &'lend i32| -> i32 { *x * 2 }),
/// );
/// ```
#[macro_export]
macro_rules! covar {($($t:tt)+) => ($crate::__covar__!(Fn, $($t)+))}
