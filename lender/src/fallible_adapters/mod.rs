use crate::TryShunt;

mod chain;
mod chunk;
mod chunky;
mod cloned;
mod convert;
mod copied;
mod cycle;
mod enumerate;
mod filter;
mod filter_map;
mod flatten;
mod fuse;
mod inspect;
mod intersperse;
mod into_fallible;
mod iter;
mod map;
mod map_err;
mod map_into_iter;
mod map_while;
mod mutate;
pub(crate) mod non_fallible_adapter;
mod owned;
mod peekable;
mod rev;
mod scan;
mod skip;
mod skip_while;
mod step_by;
mod take;
mod take_while;
mod zip;

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
