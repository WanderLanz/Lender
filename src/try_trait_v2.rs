//! Avoiding reliance on nightly feature `try_trait_v2` to implement stdlib's `Try`.
use core::{convert::Infallible, ops::ControlFlow, task::Poll};

pub(crate) type ChangeOutputType<T, V> = <<T as Try>::Residual as Residual<V>>::TryType;

pub trait Try: FromResidual {
    type Output;
    type Residual;
    fn from_output(output: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}
pub trait FromResidual<R = <Self as Try>::Residual> {
    fn from_residual(residual: R) -> Self;
}
pub trait Residual<O> {
    type TryType: Try<Output = O, Residual = Self>;
}
impl<B, C> Try for ControlFlow<B, C> {
    type Output = C;
    type Residual = ControlFlow<B, Infallible>;
    #[inline]
    fn from_output(output: Self::Output) -> Self { ControlFlow::Continue(output) }
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
            // *should* be compiled out
            _ => unreachable!(),
        }
    }
}
impl<T> Try for Option<T> {
    type Output = T;
    type Residual = Option<Infallible>;
    #[inline]
    fn from_output(output: Self::Output) -> Self { Some(output) }
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
    fn from_residual(_residual: Option<Infallible>) -> Self { None }
}
impl<T, E> Try for Result<T, E> {
    type Output = T;
    type Residual = Result<Infallible, E>;
    #[inline]
    fn from_output(output: Self::Output) -> Self { Ok(output) }
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
            // *should* be compiled out
            _ => unreachable!(),
        }
    }
}
impl<T, E> Try for Poll<Result<T, E>> {
    type Output = Poll<T>;
    type Residual = Result<Infallible, E>;
    #[inline]
    fn from_output(c: Self::Output) -> Self { c.map(Ok) }
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
            // *should* be compiled out
            _ => unreachable!(),
        }
    }
}
impl<T, E> Try for Poll<Option<Result<T, E>>> {
    type Output = Poll<Option<T>>;
    type Residual = Result<Infallible, E>;

    #[inline]
    fn from_output(c: Self::Output) -> Self { c.map(|x| x.map(Ok)) }

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
            // *should* be compiled out
            _ => unreachable!(),
        }
    }
}
