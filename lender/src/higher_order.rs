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

/// Higher-Kinded Associated Output `FnOnce`, where `Output` (B) is with lifetime `'b`.
pub trait FnOnceHKA<'b, A>: FnOnce(A) -> <Self as FnOnceHKA<'b, A>>::B {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnOnce(A) -> B> FnOnceHKA<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output `FnMut`, where `Output` (B) is with lifetime `'b`.
pub trait FnMutHKA<'b, A>: FnMut(A) -> <Self as FnMutHKA<'b, A>>::B {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnMut(A) -> B> FnMutHKA<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output `FnMut`, where `Output` (`Option<B>`) is with lifetime `'b`.
pub trait FnMutHKAOpt<'b, A>: FnMut(A) -> Option<<Self as FnMutHKAOpt<'b, A>>::B> {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnMut(A) -> Option<B>> FnMutHKAOpt<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output `FnOnce`, where `Output` (`Result<B, E>`)
/// has output type `B` with lifetime `'b`.
pub trait FnOnceHKARes<'b, A, E>: FnOnce(A) -> Result<<Self as FnOnceHKARes<'b, A, E>>::B, E> {
    type B: 'b;
}
impl<'b, A, B: 'b, E, F: FnOnce(A) -> Result<B, E>> FnOnceHKARes<'b, A, E> for F {
    type B = B;
}

/// Higher-Kinded Associated Output `FnMut`, where `Output` (`Result<B, E>`)
/// has output type `B` with lifetime `'b`.
pub trait FnMutHKARes<'b, A, E>: FnMut(A) -> Result<<Self as FnMutHKARes<'b, A, E>>::B, E> {
    type B: 'b;
}
impl<'b, A, B: 'b, E, F: FnMut(A) -> Result<B, E>> FnMutHKARes<'b, A, E> for F {
    type B = B;
}

/// Higher-Kinded Associated Output `FnMut`, where `Output` (`Result<Option<B>, E>`)
/// has output type `B` with lifetime `'b`.
pub trait FnMutHKAResOpt<'b, A, E>: FnMut(A) -> Result<Option<<Self as FnMutHKAResOpt<'b, A, E>>::B>, E> {
    type B: 'b;
}
impl<'b, A, B: 'b, E, F: FnMut(A) -> Result<Option<B>, E>> FnMutHKAResOpt<'b, A, E> for F {
    type B = B;
}

/// Not meant to be called directly. A modified version of [`higher-order-closure`](https://crates.io/crates/higher-order-closure)'s `higher_order_closure` macro to use any `Fn` trait.
#[doc(hidden)]
#[macro_export]
macro_rules! __hrc__ {(
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
)}

/// Higher-Rank Closure (FnOnce) macro that replaces the `closure_lifetime_binder` feature for stable.
///
/// This is a modified version of [`higher-order-closure`](https://crates.io/crates/higher-order-closure)'s `higher_order_closure` macro.
#[macro_export]
macro_rules! hrc_once {($($t:tt)+) => ($crate::__hrc__!(FnOnce, $($t)+))}

/// Higher-Rank Closure (FnMut)  macro that replaces the `closure_lifetime_binder` feature for stable.
///
/// This is a modified version of [`higher-order-closure`](https://crates.io/crates/higher-order-closure)'s `higher_order_closure` macro.
#[macro_export]
macro_rules! hrc_mut {($($t:tt)+) => ($crate::__hrc__!(FnMut, $($t)+))}

/// Higher-Rank Closure (Fn) macro that replaces the `closure_lifetime_binder` feature for stable.
///
/// This is a modified version of [`higher-order-closure`](https://crates.io/crates/higher-order-closure)'s `higher_order_closure` macro.
#[macro_export]
macro_rules! hrc {($($t:tt)+) => ($crate::__hrc__!(Fn, $($t)+))}
