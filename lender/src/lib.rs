#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

#[cfg(doctest)]
#[allow(non_camel_case_types)]
/// ```rust,compile_fail
/// struct WindowsMut<'a, T> {
///     inner: &'a mut [T],
///     begin: usize,
///     len: usize,
/// }
/// impl<'a, T> Iterator for WindowsMut<'a, T> {
///     type Item = &mut [T];
///
///     fn next(&mut self) -> Option<Self::Item> {
///         let begin = self.begin;
///         self.begin = self.begin.saturating_add(1);
///         self.inner.get_mut(begin..begin + self.len)
///     }
/// }
/// ```
pub struct _Lender_Doctest_Sanity_Check;

#[cfg(feature = "derive")]
pub use lender_derive::for_;

#[doc(hidden)]
#[allow(private_bounds)]
pub trait ImplBound: ImplBoundPriv {}
#[doc(hidden)]
pub(crate) trait ImplBoundPriv {}
impl<T: ?Sized + ImplBoundPriv> ImplBound for T {}
#[doc(hidden)]
pub struct Ref<'lend, T: ?Sized>(&'lend T);
impl<T: ?Sized> ImplBoundPriv for Ref<'_, T> {}

mod adapters;
pub use adapters::*;
mod fallible_adapters;
pub use fallible_adapters::*;
mod traits;
pub use traits::*;
pub mod higher_order;
pub use higher_order::Covar;
mod sources;
pub use sources::*;
mod fallible_sources;
pub use fallible_sources::*;
pub use stable_try_trait_v2 as try_trait_v2;

pub mod prelude {
    #[cfg(feature = "derive")]
    pub use lender_derive::for_;

    pub use crate::{
        Covar, CovariantFallibleLending, CovariantLending, DoubleEndedFallibleLender,
        DoubleEndedLender,
        ExactSizeFallibleLender, ExactSizeLender, ExtendFallibleLender, ExtendLender,
        FallibleIteratorExt, FallibleLend, FallibleLender, FallibleLending, FromFallibleLender,
        FromLender, FusedFallibleLender, FusedLender, IntoFallibleIteratorExt, IntoFallibleLender,
        IntoIteratorExt, IntoLender, IteratorExt, Lend, Lender, Lending, ProductFallibleLender,
        ProductLender, SumFallibleLender, SumLender, WindowsMutExt, check_covariance,
        check_covariance_fallible, covariant_fallible_lend, covariant_lend, fallible_lend,
        covar, covar_mut, covar_once, lend, unsafe_assume_covariance,
        unsafe_assume_covariance_fallible,
    };
}
