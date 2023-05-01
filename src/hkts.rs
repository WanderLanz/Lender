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

mod hkfns {
    mod with_arg {
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
        /// Higher-Kinded Associated Output `Fn`, where `Output` (B) is with lifetime `'b`.
        pub trait FnHKA<'b, A>: Fn(A) -> <Self as FnHKA<'b, A>>::B {
            type B: 'b;
        }
        impl<'b, A, B: 'b, F: Fn(A) -> B> FnHKA<'b, A> for F {
            type B = B;
        }
    }
    pub use with_arg::*;
    mod no_arg {
        pub trait FnOnceHK<'b>: FnOnce() -> <Self as FnOnceHK<'b>>::B {
            type B: 'b;
        }
        impl<'b, B: 'b, F: FnOnce() -> B> FnOnceHK<'b> for F {
            type B = B;
        }
        pub trait FnMutHK<'b>: FnMut() -> <Self as FnMutHK<'b>>::B {
            type B: 'b;
        }
        impl<'b, B: 'b, F: FnMut() -> B> FnMutHK<'b> for F {
            type B = B;
        }
        pub trait FnHK<'b>: Fn() -> <Self as FnHK<'b>>::B {
            type B: 'b;
        }
        impl<'b, B: 'b, F: Fn() -> B> FnHK<'b> for F {
            type B = B;
        }
    }
    pub use no_arg::*;

    mod opt_arg {
        /// Higher-Kinded Associated Output `FnOnce`, where `Output` (`Option<B>`) is with lifetime `'b`.
        pub trait FnOnceHKAOpt<'b, A>: FnOnce(A) -> Option<<Self as FnOnceHKAOpt<'b, A>>::B> {
            type B: 'b;
        }
        impl<'b, A, B: 'b, F: FnOnce(A) -> Option<B>> FnOnceHKAOpt<'b, A> for F {
            type B = B;
        }
        /// Higher-Kinded Associated Output `FnMut`, where `Output` (`Option<B>`) is with lifetime `'b`.
        pub trait FnMutHKAOpt<'b, A>: FnMut(A) -> Option<<Self as FnMutHKAOpt<'b, A>>::B> {
            type B: 'b;
        }
        impl<'b, A, B: 'b, F: FnMut(A) -> Option<B>> FnMutHKAOpt<'b, A> for F {
            type B = B;
        }
        /// Higher-Kinded Associated Output `Fn`, where `Output` (`Option<B>`) is with lifetime `'b`.
        pub trait FnHKAOpt<'b, A>: Fn(A) -> Option<<Self as FnHKAOpt<'b, A>>::B> {
            type B: 'b;
        }
        impl<'b, A, B: 'b, F: Fn(A) -> Option<B>> FnHKAOpt<'b, A> for F {
            type B = B;
        }
    }
    pub use opt_arg::*;
    mod opt_narg {
        pub trait FnOnceHKOpt<'b>: FnOnce() -> Option<<Self as FnOnceHKOpt<'b>>::B> {
            type B: 'b;
        }
        impl<'b, B: 'b, F: FnOnce() -> Option<B>> FnOnceHKOpt<'b> for F {
            type B = B;
        }
        pub trait FnMutHKOpt<'b>: FnMut() -> Option<<Self as FnMutHKOpt<'b>>::B> {
            type B: 'b;
        }
        impl<'b, B: 'b, F: FnMut() -> Option<B>> FnMutHKOpt<'b> for F {
            type B = B;
        }
        pub trait FnHKOpt<'b>: Fn() -> Option<<Self as FnHKOpt<'b>>::B> {
            type B: 'b;
        }
        impl<'b, B: 'b, F: Fn() -> Option<B>> FnHKOpt<'b> for F {
            type B = B;
        }
    }
    pub use opt_narg::*;
}
pub use hkfns::*;

/// Higher-Kinded Generic Output `FnOnce`, where `Output` (B) is with lifetime `'b`.
pub trait FnOnceHKG<'b, A, B: 'b>: FnOnce(A) -> B {}
impl<'b, A, B: 'b, F: FnOnce(A) -> B> FnOnceHKG<'b, A, B> for F {}
/// Higher-Kinded Generic Output `FnMut`, where `Output` (B) is with lifetime `'b`.
pub trait FnMutHKG<'b, A, B: 'b>: FnMut(A) -> B {}
impl<'b, A, B: 'b, F: FnMut(A) -> B> FnMutHKG<'b, A, B> for F {}
/// Higher-Kinded Generic Output `Fn`, where `Output` (B) is with lifetime `'b`.
pub trait FnHKG<'b, A, B: 'b>: Fn(A) -> B {}
impl<'b, A, B: 'b, F: Fn(A) -> B> FnHKG<'b, A, B> for F {}

// TODO # Higher-Kinded Try (HKTry) where Output and Residual have lifetime `'a`
