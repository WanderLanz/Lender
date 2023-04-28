use core::ops::ControlFlow;

mod chain;
mod chunk;
mod cloned;
mod copied;
mod cycle;
mod enumerate;
mod filter;
mod filter_map;
mod flatten;
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
    chain::Chain,
    chunk::Chunk,
    cloned::Cloned,
    copied::Copied,
    cycle::Cycle,
    enumerate::Enumerate,
    filter::Filter,
    filter_map::FilterMap,
    flatten::{FlatMap, Flatten},
    fuse::Fuse,
    inspect::Inspect,
    iter::Iter,
    map::Map,
    map_while::MapWhile,
    mutate::Mutate,
    owned::Owned,
    peekable::Peekable,
    rev::Rev,
    scan::Scan,
    skip::Skip,
    skip_while::SkipWhile,
    step_by::StepBy,
    take::Take,
    take_while::TakeWhile,
    zip::Zip,
};
use crate::{try_trait_v2::Try, Lender, Lending}; // Not from std::iter

// pub use zip::{TrustedRandomAccess, TrustedRandomAccessNoCoerce};

/// Turns a `Lender`, where `Lend` implements `Try`, into a `Lender` of `<Lend as Try>::Output`.
pub struct TryShunt<'this, L: Lender>
where
    for<'all> <L as Lending<'all>>::Lend: Try,
{
    pub(crate) lender: L,
    // needs to be an elevated lifetime, because the residual is returned from the closure in case of a break.
    pub(crate) residual: &'this mut Option<<<L as Lending<'this>>::Lend as Try>::Residual>,
}
impl<'this, L: Lender> TryShunt<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Try,
{
    pub(crate) fn new(lender: L, residual: &'this mut Option<<<L as Lending<'this>>::Lend as Try>::Residual>) -> Self {
        Self { lender, residual }
    }
}
impl<'lend, 'this, L: Lender> Lending<'lend> for TryShunt<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Try,
{
    type Lend = <<L as Lending<'lend>>::Lend as Try>::Output;
}
impl<'this, L: Lender> Lender for TryShunt<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Try,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if let Some(x) = self.lender.next() {
            match x.branch() {
                ControlFlow::Continue(x) => return Some(x),
                ControlFlow::Break(x) => {
                    // SAFETY: we ensure that `residual` is not accessed after this point as a normal lend value with `Break`.
                    *self.residual = Some(unsafe {
                        core::mem::transmute::<
                            <<L as Lending<'_>>::Lend as Try>::Residual,
                            <<L as Lending<'this>>::Lend as Try>::Residual,
                        >(x)
                    });
                }
            }
        }
        None
    }
}
// /// Needs some work before it can be used as intended.
// pub(crate) fn try_process<'call, L, F, U>(lender: L, mut f: F) -> ChangeOutputType<<L as Lending<'call>>::Lend, U>
// where
//     L: Lender,
//     for<'all> <L as Lending<'all>>::Lend: Try,
//     for<'all> <<L as Lending<'all>>::Lend as Try>::Residual: Residual<U>,
//     for<'all> F: FnMut(TryShunt<'all, L>) -> U,
// {
//     let mut residual = None;
//     // SAFETY: we ensure that `f` does not have access to `residual`.
//     let reborrow = unsafe { &mut *(&mut residual as *mut _) };
//     let shunt = TryShunt { lender, residual: reborrow };
//     let value = f(shunt);
//     match residual {
//         Some(r) => FromResidual::from_residual(r),
//         None => Try::from_output(value),
//     }
// }
