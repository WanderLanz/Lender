// use core::ops::ControlFlow;

// use crate::*;

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
mod iter;
mod map;
mod map_while;
mod mutate;
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

// pub use by_ref_sized::ByRefSized;
pub use intersperse::{Intersperse, IntersperseWith};
pub use zip::zip;

pub use self::{
    chain::Chain, chunk::Chunk, cloned::Cloned, copied::Copied, cycle::Cycle, enumerate::Enumerate, filter::Filter,
    filter_map::FilterMap, /* flatten::FlatMap, flatten::Flatten, */ fuse::Fuse, inspect::Inspect, map::Map,
    map_while::MapWhile, peekable::Peekable, rev::Rev, scan::Scan, skip::Skip, skip_while::SkipWhile, step_by::StepBy,
    take::Take, take_while::TakeWhile, zip::Zip,
};
pub use self::{iter::Iter, mutate::Mutate, owned::Owned}; // Not from std::iter

// pub use zip::{TrustedRandomAccess, TrustedRandomAccessNoCoerce};

// /// Turns a `Lender`, where `Lend` implements `Try`, into a `Lender` of `<Lend as Try>::Output`.
// pub(crate) struct TryShunt<'this, L, R> {
//     lender: L,
//     residual: &'this mut Option<R>,
// }
// impl<'lend, 'this, L, R> Lending<'lend> for TryShunt<'this, L, R>
// where
//     L: Lender + 'this,
//     <L as Lending<'lend>>::Lend: Try<Residual = R>,
// {
//     type Lend = <<L as Lending<'lend>>::Lend as Try>::Output;
// }
// impl<'this, L, R> Lender for TryShunt<'this, L, R>
// where
//     L: Lender + 'this,
//     for<'all> <L as Lending<'all>>::Lend: Try<Residual = R>,
// {
//     fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
//         if let Some(x) = self.lender.next() {
//             match x.branch() {
//                 ControlFlow::Continue(x) => return Some(x),
//                 ControlFlow::Break(x) => {
//                     *self.residual = Some(x);
//                 }
//             }
//         }
//         None
//     }
// }
// /// Needs some work before it can be used as intended.
// pub(crate) fn try_process<'call, L, T, R, F, U: 'call>(
//     lender: &mut L,
//     mut f: F,
// ) -> ChangeOutputType<<L as Lending<'call>>::Lend, U>
// where
//     L: Lender,
//     for<'all> <L as Lending<'all>>::Lend: Try<Output = T, Residual = R>,
//     for<'all> F: FnMut(TryShunt<'all, &'all mut L, R>) -> U,
//     R: Residual<U>,
// {
//     let mut residual = None;
//     let shunt = TryShunt { lender, residual: &mut residual };
//     let value = f(shunt);
//     match residual {
//         Some(r) => FromResidual::from_residual(r),
//         None => Try::from_output(value),
//     }
// }

// pub(crate) struct ByRef<'this, L>(pub(crate) &'this mut L);
// impl<'lend, 'this, L> Lending<'lend> for ByRef<'this, L>
// where
//     L: Lender,
// {
//     type Lend = <L as Lending<'lend>>::Lend;
// }
// impl<'this, L> Lender for ByRef<'this, L>
// where
//     L: Lender,
// {
//     fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.0.next() }
// }
