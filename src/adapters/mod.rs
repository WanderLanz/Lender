use core::{marker::PhantomData, ops::ControlFlow};

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
use crate::{
    empty,
    try_trait_v2::{ChangeOutputType, FromResidual, Residual, Try},
    Empty, ExtendLender, IntoLender, Lender, Lending, TupleLend,
};

// pub use zip::{TrustedRandomAccess, TrustedRandomAccessNoCoerce};

/// Private adapter. Turns a `Lender`, where `Lend` implements `Try`, into a `Lender` of `<Lend as Try>::Output`.
pub struct TryShunt<'this, L: Lender>
where
    for<'all> <L as Lending<'all>>::Lend: Try,
{
    lender: L,
    // needs to be an elevated lifetime, because the residual is returned from the closure in case of a break.
    residual: &'this mut Option<<<L as Lending<'this>>::Lend as Try>::Residual>,
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
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
pub(crate) fn try_process<'a, L, F, U>(lender: L, mut f: F) -> ChangeOutputType<<L as Lending<'a>>::Lend, U>
where
    L: Lender + 'a,
    for<'all> <L as Lending<'all>>::Lend: Try,
    for<'all> <<L as Lending<'all>>::Lend as Try>::Residual: Residual<U>,
    F: FnMut(TryShunt<'a, L>) -> U,
{
    let mut residual = None;
    // SAFETY: we ensure that `f` does not have access to `residual`.
    let reborrow = unsafe { &mut *(&mut residual as *mut _) };
    let shunt = TryShunt { lender, residual: reborrow };
    let value = f(shunt);
    match residual {
        Some(r) => FromResidual::from_residual(r),
        None => Try::from_output(value),
    }
}

/// Private adapter. Marks a `Lender`, where `Lend` implements `TupleLend`, as a `Lender` of `<Lend as TupleLend>::First`.
pub struct FirstShunt<L>(PhantomData<L>);
/// Private adapter. Marks a `Lender`, where `Lend` implements `TupleLend`, as a `Lender` of `<Lend as TupleLend>::Second`.
pub struct SecondShunt<L>(PhantomData<L>);
impl<'lend, L: Lender> Lending<'lend> for FirstShunt<L>
where
    for<'all> <L as Lending<'all>>::Lend: TupleLend<'all>,
{
    type Lend = <<L as Lending<'lend>>::Lend as TupleLend<'lend>>::First;
}
impl<'lend, L: Lender> Lending<'lend> for SecondShunt<L>
where
    for<'all> <L as Lending<'all>>::Lend: TupleLend<'all>,
{
    type Lend = <<L as Lending<'lend>>::Lend as TupleLend<'lend>>::Second;
}
impl<L: Lender> IntoLender for FirstShunt<L>
where
    for<'all> <L as Lending<'all>>::Lend: TupleLend<'all>,
{
    type Lender = Empty<Self>;
    fn into_lender(self) -> <Self as IntoLender>::Lender { empty() }
}
impl<L: Lender> IntoLender for SecondShunt<L>
where
    for<'all> <L as Lending<'all>>::Lend: TupleLend<'all>,
{
    type Lender = Empty<Self>;
    fn into_lender(self) -> <Self as IntoLender>::Lender { empty() }
}

pub(crate) fn unzip<L, ExtA, ExtB>(mut lender: L) -> (ExtA, ExtB)
where
    L: Sized + Lender,
    for<'all> <L as Lending<'all>>::Lend: TupleLend<'all>,
    ExtA: Default + ExtendLender<FirstShunt<L>>,
    ExtB: Default + ExtendLender<SecondShunt<L>>,
{
    let mut a = ExtA::default();
    let mut b = ExtB::default();
    let sz = lender.size_hint().0;
    if sz > 0 {
        a.extend_lender_reserve(sz);
        b.extend_lender_reserve(sz);
    }
    while let Some(lend) = lender.next() {
        let (x, y) = lend.tuple_lend();
        a.extend_lender_one(x);
        b.extend_lender_one(y);
    }
    (a, b)
}
