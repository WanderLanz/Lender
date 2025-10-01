use crate::*;

/// Documentation is incomplete. Refer to [`core::iter::FusedIterator`] for more information
pub trait FusedLender: Lender {}
impl<L: FusedLender> FusedLender for &mut L {}

/// Marker trait that ensures that a fallible lender will always continue to
/// yield Ok(None) once it has already returned Ok(None) OR Err(_).
pub trait FusedFallibleLender: FallibleLender {}
impl<L> FusedFallibleLender for &mut L where L: FusedFallibleLender {}
