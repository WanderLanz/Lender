mod chain;
mod chunk;
mod cloned;
mod copied;
mod cycle;
mod enumerate;
mod filter;
mod filter_map;
mod fuse;
mod inspect;
mod intersperse;
mod map;
mod map_while;
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

// std internals
// pub use self::by_ref_sized::ByRefSized;

use core::ops::ControlFlow;

// std unstable
pub use self::intersperse::{Intersperse, IntersperseWith};
// std unstable
// pub use self::zip::{TrustedRandomAccess, TrustedRandomAccessNoCoerce};
pub use self::zip::zip;
pub use self::{
    chain::Chain, chunk::Chunk, cloned::Cloned, copied::Copied, cycle::Cycle, enumerate::Enumerate, filter::Filter,
    filter_map::FilterMap, /* flatten::FlatMap, flatten::Flatten, */ fuse::Fuse, inspect::Inspect, map::Map,
    map_while::MapWhile, owned::Owned, peekable::Peekable, rev::Rev, scan::Scan, skip::Skip, skip_while::SkipWhile,
    step_by::StepBy, take::Take, take_while::TakeWhile, zip::Zip,
};
use crate::{
    try_trait_v2::{ChangeOutputType, FromResidual, Residual, Try},
    Lender, Lending,
};

/// Turns a `Lender`, where `Lend` implements `Try`, into a `Lender` of `<Lend as Try>::Output`.
pub(crate) struct TryShunt<'this, L, R> {
    lender: L,
    residual: &'this mut Option<R>,
}
impl<'lend, 'this, L, R> Lending<'lend> for TryShunt<'this, L, R>
where
    L: Lender + 'this,
    <L as Lending<'lend>>::Lend: Try<Residual = R>,
{
    type Lend = <<L as Lending<'lend>>::Lend as Try>::Output;
}
impl<'this, L, R> Lender for TryShunt<'this, L, R>
where
    L: Lender + 'this,
    for<'all> <L as Lending<'all>>::Lend: Try<Residual = R>,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if let Some(x) = self.lender.next() {
            match x.branch() {
                ControlFlow::Continue(x) => return Some(x),
                ControlFlow::Break(x) => {
                    *self.residual = Some(x);
                }
            }
        }
        None
    }
}
pub(crate) fn try_process<'call, L, T, R, F, U>(lender: L, mut f: F) -> ChangeOutputType<<L as Lending<'call>>::Lend, U>
where
    L: Lender,
    for<'all> <L as Lending<'all>>::Lend: 'all + Try<Output = T, Residual = R>,
    for<'all> F: FnMut(TryShunt<'all, L, R>) -> U,
    R: Residual<U>,
{
    let mut residual = None;
    let shunt = TryShunt { lender, residual: &mut residual };
    let value = f(shunt);
    match residual {
        Some(r) => FromResidual::from_residual(r),
        None => Try::from_output(value),
    }
}

pub(crate) struct ByRef<'this, L>(pub(crate) &'this mut L);
impl<'lend, 'this, L> Lending<'lend> for ByRef<'this, L>
where
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<'this, L> Lender for ByRef<'this, L>
where
    L: Lender,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.0.next() }
}
