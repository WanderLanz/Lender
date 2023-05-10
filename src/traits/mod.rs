mod accum;
mod collect;
mod double_ended;
mod exact_size;
mod lender;
mod marker;

pub use self::{
    accum::{ProductLender, SumLender},
    collect::{ExtendLender, FromLender, IntoLender},
    double_ended::DoubleEndedLender,
    exact_size::ExactSizeLender,
    lender::{Lender, Lending},
    marker::FusedLender,
};

/// Marker trait for tuple lends, used by [`Lender::unzip()`].
pub trait TupleLend<'a> {
    type First: 'a;
    type Second: 'a;
    fn tuple_lend(self) -> (Self::First, Self::Second);
}
impl<'a, A: 'a, B: 'a> TupleLend<'a> for (A, B) {
    type First = A;
    type Second = B;
    #[inline(always)]
    fn tuple_lend(self) -> (Self::First, Self::Second) { (self.0, self.1) }
}
impl<'a, A, B> TupleLend<'a> for &'a (A, B) {
    type First = &'a A;
    type Second = &'a B;
    #[inline(always)]
    fn tuple_lend(self) -> (Self::First, Self::Second) { (&self.0, &self.1) }
}
impl<'a, A, B> TupleLend<'a> for &'a mut (A, B) {
    type First = &'a mut A;
    type Second = &'a mut B;
    #[inline(always)]
    fn tuple_lend(self) -> (Self::First, Self::Second) { (&mut self.0, &mut self.1) }
}
