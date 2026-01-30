//! A simple stable replacement for the unstable `Try`, `FromResidual`, and `Residual` traits under the `try_trait_v2` feature
#![no_std]
use core::{convert::Infallible, ops::ControlFlow, task::Poll};
#[doc(hidden)]
pub mod __ {
    pub use core;
}

/// A simple macro emulating the `?` operator for this crate's `Try` types,
/// useful for contexts generic over this crate's `Try` types where the `?` operator is not usable.
#[macro_export]
macro_rules! try_ {
    ($expr:expr $(,)?) => {
        match $crate::Try::branch($expr) {
            $crate::__::core::ops::ControlFlow::Continue(o) => o,
            $crate::__::core::ops::ControlFlow::Break(r) => {
                return $crate::FromResidual::from_residual(r)
            }
        }
    };
}

/// see [`::core::ops::Try`]
pub trait Try: FromResidual {
    /// see [`::core::ops::Try::Output`]
    type Output;

    /// see [`::core::ops::Try::Residual`]
    type Residual;

    /// Constructs the type from its `Output` type.
    ///
    /// see [`::core::ops::Try::from_output()`]
    fn from_output(output: Self::Output) -> Self;

    /// see [`::core::ops::Try::branch()`]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

/// Used to specify which residuals can be converted into which [`core::ops::Try`] types.
///
/// see [`::core::ops::FromResidual`]
pub trait FromResidual<R = <Self as Try>::Residual> {
    /// Constructs the type from a compatible `Residual` type.
    ///
    /// see [`::core::ops::FromResidual::from_residual()`]
    fn from_residual(residual: R) -> Self;
}

/// Allows retrieving the canonical type implementing [`Try`] that has this type
/// as its residual and allows it to hold an `O` as its output.
///
/// see [`::core::ops::Residual`]
pub trait Residual<O> {
    /// The "return" type of this meta-function.
    type TryType: Try<Output = O, Residual = Self>;
}

pub type ChangeOutputType<T, V> = <<T as Try>::Residual as Residual<V>>::TryType;

/// core's internal helper types, not part of the public try_trait_v2 API, but exported here for convenience
pub mod internal {
    use super::*;
    /// An adapter for implementing non-try methods via the `Try` implementation.
    #[repr(transparent)]
    pub struct NeverShortCircuit<T>(pub T);

    impl<T> NeverShortCircuit<T> {
        /// Wraps a unary function to produce one that wraps the output into a `NeverShortCircuit`.
        #[inline]
        pub fn wrap_mut_1<A>(mut f: impl FnMut(A) -> T) -> impl FnMut(A) -> NeverShortCircuit<T> {
            move |a| NeverShortCircuit(f(a))
        }
        #[inline]
        pub fn wrap_mut_2<A, B>(mut f: impl FnMut(A, B) -> T) -> impl FnMut(A, B) -> Self {
            move |a, b| NeverShortCircuit(f(a, b))
        }
    }
    pub enum NeverShortCircuitResidual {}
    impl<T> Try for NeverShortCircuit<T> {
        type Output = T;
        type Residual = NeverShortCircuitResidual;
        #[inline]
        fn branch(self) -> ControlFlow<NeverShortCircuitResidual, T> {
            ControlFlow::Continue(self.0)
        }
        #[inline]
        fn from_output(x: T) -> Self {
            NeverShortCircuit(x)
        }
    }
    impl<T> FromResidual for NeverShortCircuit<T> {
        #[inline]
        fn from_residual(never: NeverShortCircuitResidual) -> Self {
            match never {}
        }
    }
    impl<T> Residual<T> for NeverShortCircuitResidual {
        type TryType = NeverShortCircuit<T>;
    }
}

impl<B, C> Try for ControlFlow<B, C> {
    type Output = C;
    type Residual = ControlFlow<B, Infallible>;
    #[inline]
    fn from_output(output: Self::Output) -> Self {
        ControlFlow::Continue(output)
    }
    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            ControlFlow::Continue(c) => ControlFlow::Continue(c),
            ControlFlow::Break(b) => ControlFlow::Break(ControlFlow::Break(b)),
        }
    }
}
impl<B, C> FromResidual for ControlFlow<B, C> {
    #[inline]
    fn from_residual(residual: ControlFlow<B, Infallible>) -> Self {
        match residual {
            ControlFlow::Break(b) => ControlFlow::Break(b),
            ControlFlow::Continue(infallible) => match infallible {},
        }
    }
}
impl<B, C> Residual<C> for ControlFlow<B, Infallible> {
    type TryType = ControlFlow<B, C>;
}

impl<T> Try for Option<T> {
    type Output = T;
    type Residual = Option<Infallible>;
    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Some(output)
    }
    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Some(v) => ControlFlow::Continue(v),
            None => ControlFlow::Break(None),
        }
    }
}
impl<T> FromResidual for Option<T> {
    #[inline]
    fn from_residual(_: Option<Infallible>) -> Self {
        None
    }
}
impl<T> Residual<T> for Option<Infallible> {
    type TryType = Option<T>;
}

impl<T, E> Try for Result<T, E> {
    type Output = T;
    type Residual = Result<Infallible, E>;
    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Ok(output)
    }
    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(v) => ControlFlow::Continue(v),
            Err(e) => ControlFlow::Break(Err(e)),
        }
    }
}
impl<T, E, F: From<E>> FromResidual<Result<Infallible, E>> for Result<T, F> {
    #[inline]
    #[track_caller]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Err(e) => Err(From::from(e)),
            Ok(infallible) => match infallible {},
        }
    }
}
impl<T, E> Residual<T> for Result<Infallible, E> {
    type TryType = Result<T, E>;
}

impl<T, E> Try for Poll<Result<T, E>> {
    type Output = Poll<T>;
    type Residual = Result<Infallible, E>;
    #[inline]
    fn from_output(c: Self::Output) -> Self {
        c.map(Ok)
    }
    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Poll::Ready(Ok(x)) => ControlFlow::Continue(Poll::Ready(x)),
            Poll::Ready(Err(e)) => ControlFlow::Break(Err(e)),
            Poll::Pending => ControlFlow::Continue(Poll::Pending),
        }
    }
}
impl<T, E, F: From<E>> FromResidual<Result<Infallible, E>> for Poll<Result<T, F>> {
    #[inline]
    fn from_residual(x: Result<Infallible, E>) -> Self {
        match x {
            Err(e) => Poll::Ready(Err(From::from(e))),
            Ok(infallible) => match infallible {},
        }
    }
}

impl<T, E> Try for Poll<Option<Result<T, E>>> {
    type Output = Poll<Option<T>>;
    type Residual = Result<Infallible, E>;

    #[inline]
    fn from_output(c: Self::Output) -> Self {
        c.map(|x| x.map(Ok))
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Poll::Ready(Some(Ok(x))) => ControlFlow::Continue(Poll::Ready(Some(x))),
            Poll::Ready(Some(Err(e))) => ControlFlow::Break(Err(e)),
            Poll::Ready(None) => ControlFlow::Continue(Poll::Ready(None)),
            Poll::Pending => ControlFlow::Continue(Poll::Pending),
        }
    }
}
impl<T, E, F: From<E>> FromResidual<Result<Infallible, E>> for Poll<Option<Result<T, F>>> {
    #[inline]
    fn from_residual(x: Result<Infallible, E>) -> Self {
        match x {
            Err(e) => Poll::Ready(Some(Err(From::from(e)))),
            Ok(infallible) => match infallible {},
        }
    }
}
