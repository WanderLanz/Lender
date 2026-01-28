use crate::TryShunt;

mod convert;
mod flatten;
mod intersperse;
mod into_fallible;
mod map_err;
pub(crate) mod non_fallible_adapter;
mod peekable;

pub use convert::Convert;
pub use flatten::{FlatMap as FallibleFlatMap, Flatten as FallibleFlatten};
pub use intersperse::{
    Intersperse as FallibleIntersperse, IntersperseWith as FallibleIntersperseWith,
};
pub use into_fallible::IntoFallible;
pub use map_err::MapErr;
pub(crate) use non_fallible_adapter::NonFallibleAdapter;
pub use peekable::Peekable as FalliblePeekable;

pub type FallibleTryShuntAdapter<'a, 'b, 'c, 'd, L> =
    TryShunt<'a, &'b mut NonFallibleAdapter<'c, &'d mut L>>;
