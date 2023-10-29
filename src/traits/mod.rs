mod accum;
mod collect;
mod double_ended;
mod exact_size;
mod ext;
mod lender;
mod marker;

pub use self::{
    accum::{ProductLender, SumLender},
    collect::{ExtendLender, FromLender, IntoLender},
    double_ended::DoubleEndedLender,
    exact_size::ExactSizeLender,
    ext::IteratorExt,
    lender::{Lend, Lender, Lending},
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
    fn tuple_lend(self) -> (Self::First, Self::Second) {
        (self.0, self.1)
    }
}
impl<'a, A, B> TupleLend<'a> for &'a (A, B) {
    type First = &'a A;
    type Second = &'a B;
    #[inline(always)]
    fn tuple_lend(self) -> (Self::First, Self::Second) {
        (&self.0, &self.1)
    }
}
impl<'a, A, B> TupleLend<'a> for &'a mut (A, B) {
    type First = &'a mut A;
    type Second = &'a mut B;
    #[inline(always)]
    fn tuple_lend(self) -> (Self::First, Self::Second) {
        (&mut self.0, &mut self.1)
    }
}

/// Internal struct used to implement [`lend!`], do not use directly.
#[doc(hidden)]
pub struct DynLendShunt<T: ?Sized>(pub T);

impl<'lend, T: ?Sized + for<'all> DynLend<'all>> Lending<'lend> for DynLendShunt<T> {
    type Lend = <T as DynLend<'lend>>::Lend;
}

/// Internal trait used to implement [`lend!`], do not use directly.
#[doc(hidden)]
pub trait DynLend<'lend> {
    type Lend;
}

/// Use lifetime `'lend` within type `$T` to create an `impl for<'lend> Lending<'lend, Lend = $T>`.
/// Uses a bug in the borrow checker which allows dyn objects to implement impossible traits.
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut empty = lender::empty::<lend!(&'lend mut [u32])>(); // <- same Lending signature as a WindowsMut over u32
/// let _: Option<&mut [u32]> = empty.next(); // => None
/// ```
#[macro_export]
macro_rules! lend {
    ($T:ty) => {
        $crate::DynLendShunt<dyn for<'lend> $crate::DynLend<'lend, Lend = $T>>
    };
}
