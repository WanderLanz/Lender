use crate::*;

/// Documentation is incomplete. Refer to [`core::iter::FusedIterator`] for more information
pub trait FusedLender: Lender {}
impl<L: FusedLender> FusedLender for &mut L {}
