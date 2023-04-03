//! These allow some lifetime elision in signatures using a HRTB and more flexible lifetime binding.
//!
//! ## Higher-Kinded Types (HKT),
//!
//! type `K` with lifetime `'a` where `K: 'a`
//! ```ignore
//! K: for<'all where K: 'all> WithLifetime<'all, T = K>
//! ```
//!
//! ## Higher-Kinded Associated Output Functions (HKAFn),
//!
//! Fn traits with associated output type `B` with lifetime `'b`
//! ```ignore
//! impl<'b, A, B: 'b, F: FnOnce(A) -> B> HKAFnOnce<'b, A> for F {
//!     type B = B;
//! }
//! ```
//! Using an associated type also allows omission of `Output` types in adapter generics, e.g. Map<L, F> vs Map<NewItemType, L, F>
//!
//! ## Higher-Kinded Generic Output Functions (HKGFn)
//!
//! Fn traits with generic output type `B` with lifetime `'b`
//! ```ignore
//! impl<'b, A, B: 'b, F: FnOnce(A) -> B> HKGFnOnce<'b, A, B> for F {}
//! ```
//! Using a generic lets the compiler infer the input and output types where it might not be able to infer the associated type of an HKAFn.

use crate::{Seal, Sealed};

/// Enforce an HKT to be with lifetime `'a` with function signature
#[inline(always)]
pub fn with_lifetime<'a, T>(hkt: T) -> <T as WithLifetime<'a>>::T
where
    T: HKT,
{
    hkt
}
/// Trait underlying HKT
pub trait WithLifetime<'lt, __Seal: Sealed = Seal<&'lt Self>> {
    type T: 'lt;
}
impl<'lt, T> WithLifetime<'lt> for T {
    type T = T;
}

/// Higher-Kinded Type, type `T` with lifetime `'a`.
pub trait HKT: for<'all> WithLifetime<'all, T = Self> {
    /// Enforce `Self` to be with lifetime `'a` with function signature
    #[inline(always)]
    fn with_lifetime<'a>(self) -> <Self as WithLifetime<'a>>::T
    where
        Self: Sized,
    {
        self
    }
}
impl<T: ?Sized> HKT for T where for<'all> Self: WithLifetime<'all, T = Self> {}

/// Higher-Kinded Associated Output `FnOnce`, where `Output` (B) is with lifetime `'b`.
pub trait HKAFnOnce<'b, A>: FnOnce(A) -> <Self as HKAFnOnce<'b, A>>::B {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnOnce(A) -> B> HKAFnOnce<'b, A> for F {
    type B = B;
}
/// Higher-Kinded Associated Output `FnMut`, where `Output` (B) is with lifetime `'b`.
pub trait HKAFnMut<'b, A>: FnMut(A) -> <Self as HKAFnMut<'b, A>>::B {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnMut(A) -> B> HKAFnMut<'b, A> for F {
    type B = B;
}
/// Higher-Kinded Associated Output `Fn`, where `Output` (B) is with lifetime `'b`.
pub trait HKAFn<'b, A>: Fn(A) -> <Self as HKAFn<'b, A>>::B {
    type B: 'b;
}
impl<'b, A, B: 'b, F: Fn(A) -> B> HKAFn<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Associated Output `FnOnce`, where `Output` (Option<B>) is with lifetime `'b`.
pub trait HKAFnOnceOpt<'b, A>: FnOnce(A) -> Option<<Self as HKAFnOnceOpt<'b, A>>::B> {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnOnce(A) -> Option<B>> HKAFnOnceOpt<'b, A> for F {
    type B = B;
}
/// Higher-Kinded Associated Output `FnMut`, where `Output` (Option<B>) is with lifetime `'b`.
pub trait HKAFnMutOpt<'b, A>: FnMut(A) -> Option<<Self as HKAFnMutOpt<'b, A>>::B> {
    type B: 'b;
}
impl<'b, A, B: 'b, F: FnMut(A) -> Option<B>> HKAFnMutOpt<'b, A> for F {
    type B = B;
}
/// Higher-Kinded Associated Output `Fn`, where `Output` (Option<B>) is with lifetime `'b`.
pub trait HKAFnOpt<'b, A>: Fn(A) -> Option<<Self as HKAFnOpt<'b, A>>::B> {
    type B: 'b;
}
impl<'b, A, B: 'b, F: Fn(A) -> Option<B>> HKAFnOpt<'b, A> for F {
    type B = B;
}

/// Higher-Kinded Generic Output `FnOnce`, where `Output` (B) is with lifetime `'b`.
pub trait HKGFnOnce<'b, A, B: 'b>: FnOnce(A) -> B {}
impl<'b, A, B: 'b, F: FnOnce(A) -> B> HKGFnOnce<'b, A, B> for F {}
/// Higher-Kinded Generic Output `FnMut`, where `Output` (B) is with lifetime `'b`.
pub trait HKGFnMut<'b, A, B: 'b>: FnMut(A) -> B {}
impl<'b, A, B: 'b, F: FnMut(A) -> B> HKGFnMut<'b, A, B> for F {}
/// Higher-Kinded Generic Output `Fn`, where `Output` (B) is with lifetime `'b`.
pub trait HKGFn<'b, A, B: 'b>: Fn(A) -> B {}
impl<'b, A, B: 'b, F: Fn(A) -> B> HKGFn<'b, A, B> for F {}

// TODO # Higher-Kinded Try (HKTry) where Output and Residual have lifetime `'a`
