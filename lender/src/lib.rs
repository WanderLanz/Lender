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

#[allow(private_bounds)]
pub trait ImplBound: ImplBoundPriv {}
pub(crate) trait ImplBoundPriv {}
impl<T: ?Sized + ImplBoundPriv> ImplBound for T {}
pub struct Ref<'lend, T: ?Sized>(&'lend T);
impl<'lend, T: ?Sized> ImplBoundPriv for Ref<'lend, T> {}

mod adapters;
pub use adapters::*;
mod traits;
pub use traits::*;
pub mod higher_order;
mod sources;
pub use sources::*;
pub mod try_trait_v2;

pub mod prelude {
    pub use crate::{
        hrc, hrc_mut, hrc_once, lend, DoubleEndedLender, ExactSizeLender, ExtendLender, FromLender, IntoLender, IteratorExt,
        Lend, Lender, Lending, WindowsMutExt,
    };
}
