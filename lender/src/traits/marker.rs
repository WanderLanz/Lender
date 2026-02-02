use crate::*;

/// The [`Lender`] version of [`core::iter::FusedIterator`].
pub trait FusedLender: Lender {}
impl<L: FusedLender> FusedLender for &mut L {}

/// Marker trait that ensures that a fallible lender will always continue to
/// yield `Ok(None)` once it has already returned `Ok(None)`. After an error,
/// the behavior is implementation-defined.
pub trait FusedFallibleLender: FallibleLender {}
impl<L> FusedFallibleLender for &mut L where L: FusedFallibleLender {}
