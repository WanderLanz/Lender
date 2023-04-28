use crate::*;

pub trait FusedLender: Lender {}
impl<L: FusedLender> FusedLender for &mut L {}
