//! # Higher-Order Types, Traits, etc.
//!
//! - Flexible function signatures, to work around function lifetime signature restrictions.
//! - Higher-Rank Closures (macro to create fn signatures as a type hint)
//!
//! If you are using nightly, it is recommended to use higher-rank closures
//! (`for<'all> |x: &'all ()| -> &'all () { x }`), which better satisfy these traits
//! without addition function signatures. (`#![feature(closure_lifetime_binder)]`).
//!
//! If you are not on nightly, you can use the [`hrc!`](`crate::hrc`), [`hrc_mut!`](`crate::hrc_mut`),
//! or [`hrc_once!`](`crate::hrc_once`) macros to create a higher-rank closure.

/// Higher-Kinded Associated Output [`FnOnce`], where `Output` (B) is with lifetime `'b`.
pub trait FnOnceHKA<'b, A>: FnOnce(A) -> <Self as FnOnceHKA<'b, A>>::B {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnOnce(A) -> B> FnOnceHKA<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnMut`], where `Output` (B) is with lifetime `'b`.
pub trait FnMutHKA<'b, A>: FnMut(A) -> <Self as FnMutHKA<'b, A>>::B {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnMut(A) -> B> FnMutHKA<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnMut`], where `Output` ([`Option<B>`](Option)) is with
/// lifetime `'b`.
pub trait FnMutHKAOpt<'b, A>: FnMut(A) -> Option<<Self as FnMutHKAOpt<'b, A>>::B> {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnMut(A) -> Option<B>> FnMutHKAOpt<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnOnce`], where `Output` ([`Result<B, E>`](Result))
/// has output type `B` with lifetime `'b`.
pub trait FnOnceHKARes<'b, A, E>: FnOnce(A) -> Result<<Self as FnOnceHKARes<'b, A, E>>::B, E> {
    type B: 'b;
}
impl<'b, A, B: 'b, E, F: FnOnce(A) -> Result<B, E>> FnOnceHKARes<'b, A, E> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnMut`], where `Output` ([`Result<B, E>`](Result))
/// has output type `B` with lifetime `'b`.
pub trait FnMutHKARes<'b, A, E>: FnMut(A) -> Result<<Self as FnMutHKARes<'b, A, E>>::B, E> {
    type B: 'b;
}
impl<'b, A, B: 'b, E, F: FnMut(A) -> Result<B, E>> FnMutHKARes<'b, A, E> for F {
    type B = B;
}

/// Higher-Kinded Associated Output [`FnMut`], where `Output` (`Result<Option<B>, E>`)
/// has output type `B` with lifetime `'b`.
pub trait FnMutHKAResOpt<'b, A, E>: FnMut(A) -> Result<Option<<Self as FnMutHKAResOpt<'b, A, E>>::B>, E> {
    type B: 'b;
}
impl<'b, A, B: 'b, E, F: FnMut(A) -> Result<Option<B>, E>> FnMutHKAResOpt<'b, A, E> for F {
    type B = B;
}

/// Not meant to be called directly. A modified version of
/// [`higher-order-closure`](https://crates.io/crates/higher-order-closure)'s
/// `higher_order_closure` macro to use any [`Fn`] trait.
#[doc(hidden)]
#[macro_export]
macro_rules! __hrc__ {
    // Case 1: With for<'lifetime> and return type - includes covariance check
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

        for<$($hr:lifetime),+ $(,)?>
        $( move $(@$move:tt)?)?
        | $($arg:tt : $Arg:ty),* $(,)?|
        -> $Ret:ty
        $body:block
    ) => (
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
                __Closure : for<$($hr),+> ::core::ops::$F($($Arg),*) -> $Ret,
                $($($($wc)*)?)?
            {
                // Covariance check: this struct has the same variance as $Ret
                // with respect to the bound lifetime(s). The PhantomData<&'a ()>
                // ensures the lifetime is used even if $Ret doesn't contain it.
                #[allow(dead_code)]
                struct __CovarCheck<$($hr),+>(
                    ::core::marker::PhantomData<($Ret, $(&$hr ()),+)>
                );

                // This function only compiles if __CovarCheck (and thus $Ret)
                // is covariant in the lifetime parameter(s).
                // Using `x` directly (not panic!) ensures the type conversion is checked.
                #[allow(dead_code)]
                fn __check_covariance<'__long: '__short, '__short>(
                    x: *const __CovarCheck<'__long>,
                ) -> *const __CovarCheck<'__short> {
                    x
                }

                f
            }

            __funnel__::<$($($($T ,)+)?)? _>
        })(
            $(move $($move)?)? |$($arg),*| $body
        )
    );

    // Case 2: Without for<> or without return type - no covariance check needed
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

        $( for<$($hr:lifetime),* $(,)?> )?
        $( move $(@$move:tt)?)?
        | $($arg:tt : $Arg:ty),* $(,)?|
        $( -> $Ret:ty)?
        $body:block
    ) => (
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
                __Closure : for<$($($hr ,)*)?> ::core::ops::$F($($Arg),*)$( -> $Ret)?,
                $($($($wc)*)?)?
            {
                f
            }

            __funnel__::<$($($($T ,)+)?)? _>
        })(
            $(move $($move)?)? |$($arg),*| $body
        )
    );
}

/// Higher-Rank Closure ([`FnOnce`]) macro for creating closures with explicit lifetime bounds.
///
/// This macro is a stable replacement for the nightly `closure_lifetime_binder` feature
/// (`for<'a> |x: &'a T| -> &'a U { ... }`). It wraps a closure and provides the necessary
/// type hints for the compiler to understand higher-rank lifetime bounds.
///
/// Use `hrc_once!` when the closure will only be called once (e.g., with [`Lender::fold`](crate::Lender::fold)).
/// For closures that may be called multiple times, use [`hrc_mut!`](crate::hrc_mut) or [`hrc!`](crate::hrc).
///
/// # Covariance Check
///
/// When a return type is specified with `for<'a>` bounds, the macro includes a compile-time
/// covariance check. This ensures that the return type is covariant in the bound lifetime,
/// which is required for soundness with lending iterators. Invariant types like
/// `Cell<&'a T>` will cause a compilation error.
///
/// # Syntax
///
/// ```text
/// hrc_once!(for<'a> |arg: Type<'a>| -> ReturnType<'a> { body })
/// hrc_once!(for<'a> move |arg: Type<'a>| -> ReturnType<'a> { body })
/// ```
///
/// # Examples
///
/// ```rust
/// use lender::prelude::*;
///
/// // Using hrc_once! with once_with (closure called exactly once)
/// let mut lender = lender::once_with(42u8,
///     hrc_once!(for<'lend> |state: &'lend mut u8| -> &'lend mut u8 {
///         *state += 1;
///         state
///     })
/// );
/// assert_eq!(lender.next(), Some(&mut 43));
/// assert_eq!(lender.next(), None);
/// ```
///
/// This is a modified version of
/// [`higher-order-closure`](https://crates.io/crates/higher-order-closure)'s
/// `higher_order_closure` macro.
#[macro_export]
macro_rules! hrc_once {($($t:tt)+) => ($crate::__hrc__!(FnOnce, $($t)+))}

/// Higher-Rank Closure ([`FnMut`]) macro for creating closures with explicit lifetime bounds.
///
/// This macro is a stable replacement for the nightly `closure_lifetime_binder` feature
/// (`for<'a> |x: &'a T| -> &'a U { ... }`). It wraps a closure and provides the necessary
/// type hints for the compiler to understand higher-rank lifetime bounds.
///
/// Use `hrc_mut!` when the closure may be called multiple times and captures mutable state
/// or needs `&mut self` semantics. This is the most commonly used variant for lender methods
/// like [`Lender::map`](crate::Lender::map), [`Lender::for_each`](crate::Lender::for_each), [`Lender::filter_map`](crate::Lender::filter_map), and [`Lender::scan`](crate::Lender::scan).
///
/// # Covariance Check
///
/// When a return type is specified with `for<'a>` bounds, the macro includes a compile-time
/// covariance check. This ensures that the return type is covariant in the bound lifetime,
/// which is required for soundness with lending iterators. Invariant types like
/// `Cell<&'a T>` will cause a compilation error.
///
/// # Syntax
///
/// ```text
/// hrc_mut!(for<'a> |arg: Type<'a>| -> ReturnType<'a> { body })
/// hrc_mut!(for<'a> move |arg: Type<'a>| -> ReturnType<'a> { body })
/// ```
///
/// # Examples
///
/// ```rust
/// use lender::prelude::*;
///
/// // Using hrc_mut! with map to transform borrowed elements
/// let mut data = [1, 2, 3, 4];
/// let mut lender = lender::windows_mut(&mut data, 2)
///     .map(hrc_mut!(for<'lend> |w: &'lend mut [i32]| -> &'lend mut i32 {
///         &mut w[0]
///     }));
/// assert_eq!(lender.next(), Some(&mut 1));
/// ```
///
/// ```rust
/// use lender::prelude::*;
///
/// // Using hrc_mut! with for_each
/// let mut data = [0, 1, 0, 0, 0, 0, 0, 0, 0];
/// lender::windows_mut(&mut data, 3)
///     .for_each(hrc_mut!(for<'lend> |w: &'lend mut [i32]| {
///         w[2] = w[0] + w[1];  // Compute Fibonacci
///     }));
/// assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
/// ```
///
/// This is a modified version of
/// [`higher-order-closure`](https://crates.io/crates/higher-order-closure)'s
/// `higher_order_closure` macro.
#[macro_export]
macro_rules! hrc_mut {($($t:tt)+) => ($crate::__hrc__!(FnMut, $($t)+))}

/// Higher-Rank Closure ([`Fn`]) macro for creating closures with explicit lifetime bounds.
///
/// This macro is a stable replacement for the nightly `closure_lifetime_binder` feature
/// (`for<'a> |x: &'a T| -> &'a U { ... }`). It wraps a closure and provides the necessary
/// type hints for the compiler to understand higher-rank lifetime bounds.
///
/// Use `hrc!` when the closure only needs shared access to its captures (`&self` semantics)
/// and may be called multiple times. In practice, [`hrc_mut!`](crate::hrc_mut) is more commonly used since
/// most lender methods require [`FnMut`].
///
/// # Covariance Check
///
/// When a return type is specified with `for<'a>` bounds, the macro includes a compile-time
/// covariance check. This ensures that the return type is covariant in the bound lifetime,
/// which is required for soundness with lending iterators. Invariant types like
/// `Cell<&'a T>` will cause a compilation error.
///
/// # Syntax
///
/// ```text
/// hrc!(for<'a> |arg: Type<'a>| -> ReturnType<'a> { body })
/// hrc!(for<'a> move |arg: Type<'a>| -> ReturnType<'a> { body })
/// ```
///
/// # Examples
///
/// ```rust
/// use lender::prelude::*;
///
/// // hrc! can be used where Fn is sufficient, but hrc_mut! also works
/// let data = vec![1, 2, 3];
/// let lender = lender::from_iter(data.iter());
/// let mapped = lender.map(hrc!(for<'lend> |x: &'lend i32| -> i32 { *x * 2 }));
/// ```
///
/// This is a modified version of
/// [`higher-order-closure`](https://crates.io/crates/higher-order-closure)'s
/// `higher_order_closure` macro.
#[macro_export]
macro_rules! hrc {($($t:tt)+) => ($crate::__hrc__!(Fn, $($t)+))}
