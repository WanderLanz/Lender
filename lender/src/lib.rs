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
impl<'lend, T: ?Sized> ImplBoundPriv for Ref<'lend, T> {}

mod adapters;
pub use adapters::*;
mod traits;
pub use traits::*;
pub mod higher_order;
mod sources;
pub use sources::*;
pub use stable_try_trait_v2 as try_trait_v2;

pub mod prelude {
    #[cfg(feature = "derive")]
    pub use lender_derive::for_;

    pub use crate::{
        from_into_iter, from_iter, hrc, hrc_mut, hrc_once, lend, DoubleEndedLender, ExactSizeLender, ExtendLender,
        FromLender, IntoIteratorExt, IntoLender, IteratorExt, Lend, Lender, Lending, WindowsMutExt,
    };
}
