//! Higher-Kinded Types (HKT), Higher-Kinded Functions (HKFn)

// # Higher-Kinded Type (HKT)

use crate::sealed::{Seal, Sealed};

/// Enforce `T` to be with lifetime `'a` with function signature
#[inline(always)]
pub fn hkt<'a, T>(hkt: T) -> <T as WithLifetime<'a>>::T
where
    T: HKT,
{
    hkt
}
pub trait WithLifetime<'lt, Bind: Sealed = Seal<&'lt Self>> {
    type T: 'lt;
}
impl<'a, T> WithLifetime<'a> for T {
    type T = T;
}

/// Higher-Kinded Type, type `T` with lifetime `'a`.
/// Not actually working as intended, but it might be used later.
pub trait HKT: for<'any> WithLifetime<'any, T = Self> {
    /// Enforce `Self` to be with lifetime `'a` with function signature
    #[inline(always)]
    fn hkt<'a>(self) -> <Self as WithLifetime<'a>>::T
    where
        Self: Sized,
    {
        self
    }
}
impl<T: ?Sized> HKT for T where for<'any> Self: 'any + WithLifetime<'any, T = Self> {}

// # Higher-Kinded Function (HKFn)

/// Higher-Kinded FnOnce, where Output is with lifetime `'ret`.
pub trait HKFnOnce<'ret, Arg>
where
    Self: FnOnce(Arg) -> <Self as HKFnOnce<'ret, Arg>>::HKOutput,
{
    type HKOutput: 'ret;
}
impl<'ret, A, B, F> HKFnOnce<'ret, A> for F
where
    F: FnOnce(A) -> B,
    B: 'ret,
{
    type HKOutput = B;
}

/// Higher-Kinded FnMut, where Output is with lifetime `'ret`.
pub trait HKFnMut<'ret, Arg>
where
    Self: HKFnOnce<'ret, Arg> + FnMut(Arg) -> <Self as HKFnOnce<'ret, Arg>>::HKOutput,
{
}
impl<'ret, A, B, F> HKFnMut<'ret, A> for F
where
    F: FnMut(A) -> B,
    B: 'ret,
{
}

/// Higher-Kinded Fn, where Output is with lifetime `'ret`.
pub trait HKFn<'ret, Arg>
where
    Self: HKFnMut<'ret, Arg> + Fn(Arg) -> <Self as HKFnOnce<'ret, Arg>>::HKOutput,
{
}
impl<'ret, A, B, F> HKFn<'ret, A> for F
where
    F: Fn(A) -> B,
    B: 'ret,
{
}

// ## HKFns where Output is an Option

/// Higher-Kinded FnOnce, where Output is an Option with lifetime `'ret`.
pub trait HKFnOnceOpt<'ret, Arg>
where
    Self: FnOnce(Arg) -> Option<<Self as HKFnOnceOpt<'ret, Arg>>::HKOutput>,
{
    type HKOutput: 'ret;
}
impl<'ret, A, B, F> HKFnOnceOpt<'ret, A> for F
where
    F: FnOnce(A) -> Option<B>,
    B: 'ret,
{
    type HKOutput = B;
}
/// Higher-Kinded FnMut, where Output is an Option with lifetime `'ret`.
pub trait HKFnMutOpt<'ret, Arg>
where
    Self: HKFnOnceOpt<'ret, Arg> + FnMut(Arg) -> Option<<Self as HKFnOnceOpt<'ret, Arg>>::HKOutput>,
{
}
impl<'ret, A, B, F> HKFnMutOpt<'ret, A> for F
where
    F: FnMut(A) -> Option<B>,
    B: 'ret,
{
}
/// Higher-Kinded Fn, where Output is an Option with lifetime `'ret`.
pub trait HKFnOpt<'ret, Arg>
where
    Self: HKFnMutOpt<'ret, Arg> + Fn(Arg) -> Option<<Self as HKFnOnceOpt<'ret, Arg>>::HKOutput>,
{
}
impl<'ret, A, B, F> HKFnOpt<'ret, A> for F
where
    F: Fn(A) -> Option<B>,
    B: 'ret,
{
}

// TODO # Higher-Kinded Try (HKTry) where Output and Residual have lifetime `'a`
