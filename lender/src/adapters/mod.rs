use core::{marker::PhantomData, ops::ControlFlow};

mod chain;
mod chunk;
mod chunky;
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
mod map_into_iter;
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
    chunky::Chunky,
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
    map_into_iter::MapIntoIter,
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
    empty, fallible_empty,
    try_trait_v2::{ChangeOutputType, FromResidual, Residual, Try},
    Empty, FallibleEmpty, ExtendLender, FallibleLend, FallibleLender, FallibleLending, IntoFallibleLender, IntoLender, Lend,
    Lender, Lending, NonFallibleAdapter, TupleLend,
};

// pub use zip::{TrustedRandomAccess, TrustedRandomAccessNoCoerce};

/// Private adapter. Turns a `Lender`, where `Lend` implements `Try`, into a `Lender` of `<Lend as Try>::Output`.
/// # Safety
/// The residual of the lender cannot outlive it, otherwise UB.
pub struct TryShunt<'this, L: Lender>
where
    for<'all> Lend<'all, L>: Try,
{
    lender: L,
    residual: &'this mut Option<<Lend<'this, L> as Try>::Residual>,
}
impl<'lend, L: Lender> Lending<'lend> for TryShunt<'_, L>
where
    for<'all> Lend<'all, L>: Try,
{
    type Lend = <Lend<'lend, L> as Try>::Output;
}
impl<'this, L: Lender> Lender for TryShunt<'this, L>
where
    for<'all> Lend<'all, L>: Try,
{
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.residual.is_some() {
            return None;
        }
        if let Some(x) = self.lender.next() {
            match x.branch() {
                ControlFlow::Continue(x) => return Some(x),
                ControlFlow::Break(x) => {
                    // SAFETY: residual is manually guaranteed to be the only lend alive
                    *self.residual = Some(unsafe {
                        core::mem::transmute::<<Lend<'_, L> as Try>::Residual, <Lend<'this, L> as Try>::Residual>(x)
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
pub(crate) fn try_process<'a, L, F, U>(lender: L, mut f: F) -> ChangeOutputType<Lend<'a, L>, U>
where
    L: Lender + 'a,
    for<'all> Lend<'all, L>: Try,
    for<'all> <Lend<'all, L> as Try>::Residual: Residual<U>,
    for<'all> F: FnMut(TryShunt<'all, L>) -> U,
{
    let mut residual = None;
    // SAFETY: residual is manually guaranteed to be the only lend alive after `f`.
    let reborrow = unsafe { &mut *(&mut residual as *mut _) };
    let shunt = TryShunt { lender, residual: reborrow };
    let value = f(shunt);
    match residual {
        Some(r) => FromResidual::from_residual(r),
        None => Try::from_output(value),
    }
}

#[doc(hidden)]
/// Private adapter. Marks a `Lender`, where `Lend` implements `TupleLend`, as a
/// `Lender` of `<Lend as TupleLend>::First`.
pub struct FirstShunt<L>(PhantomData<L>);
#[doc(hidden)]
/// Private adapter. Marks a `Lender`, where `Lend` implements `TupleLend`, as a
/// `Lender` of `<Lend as TupleLend>::Second`.
pub struct SecondShunt<L>(PhantomData<L>);
impl<'lend, L: Lender> Lending<'lend> for FirstShunt<L>
where
    for<'all> Lend<'all, L>: TupleLend<'all>,
{
    type Lend = <Lend<'lend, L> as TupleLend<'lend>>::First;
}
impl<'lend, L: Lender> Lending<'lend> for SecondShunt<L>
where
    for<'all> Lend<'all, L>: TupleLend<'all>,
{
    type Lend = <Lend<'lend, L> as TupleLend<'lend>>::Second;
}
impl<L: Lender> IntoLender for FirstShunt<L>
where
    for<'all> Lend<'all, L>: TupleLend<'all>,
{
    type Lender = Empty<Self>;
    fn into_lender(self) -> <Self as IntoLender>::Lender {
        empty()
    }
}
impl<L: Lender> IntoLender for SecondShunt<L>
where
    for<'all> Lend<'all, L>: TupleLend<'all>,
{
    type Lender = Empty<Self>;
    fn into_lender(self) -> <Self as IntoLender>::Lender {
        empty()
    }
}
impl<'lend, L: FallibleLender> FallibleLending<'lend> for FirstShunt<L>
where
    for<'all> FallibleLend<'all, L>: TupleLend<'all>,
{
    type Lend = <FallibleLend<'lend, L> as TupleLend<'lend>>::First;
}
impl<'lend, L: FallibleLender> FallibleLending<'lend> for SecondShunt<L>
where
    for<'all> FallibleLend<'all, L>: TupleLend<'all>,
{
    type Lend = <FallibleLend<'lend, L> as TupleLend<'lend>>::Second;
}
impl<L: FallibleLender> IntoFallibleLender for FirstShunt<L>
where
    for<'all> FallibleLend<'all, L>: TupleLend<'all>,
{
    type Error = L::Error;
    type FallibleLender = FallibleEmpty<L::Error, Self>;
    fn into_fallible_lender(self) -> <Self as IntoFallibleLender>::FallibleLender {
        fallible_empty()
    }
}
impl<L: FallibleLender> IntoFallibleLender for SecondShunt<L>
where
    for<'all> FallibleLend<'all, L>: TupleLend<'all>,
{
    type Error = L::Error;
    type FallibleLender = FallibleEmpty<L::Error, Self>;
    fn into_fallible_lender(self) -> <Self as IntoFallibleLender>::FallibleLender {
        fallible_empty()
    }
}

pub(crate) fn unzip<L, ExtA, ExtB>(mut lender: L) -> (ExtA, ExtB)
where
    L: Sized + Lender,
    for<'all> Lend<'all, L>: TupleLend<'all>,
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

pub(crate) fn fallible_unzip<L, ExtA, ExtB>(mut lender: L) -> Result<(ExtA, ExtB), L::Error>
where
    L: Sized + FallibleLender,
    for<'all> FallibleLend<'all, L>: TupleLend<'all>,
    ExtA:
        Default + for<'this> ExtendLender<NonFallibleAdapter<'this, <FirstShunt<L> as IntoFallibleLender>::FallibleLender>>,
    ExtB:
        Default + for<'this> ExtendLender<NonFallibleAdapter<'this, <SecondShunt<L> as IntoFallibleLender>::FallibleLender>>,
{
    let mut a = ExtA::default();
    let mut b = ExtB::default();
    let sz = lender.size_hint().0;
    if sz > 0 {
        a.extend_lender_reserve(sz);
        b.extend_lender_reserve(sz);
    }
    while let Some(lend) = lender.next()? {
        let (x, y) = lend.tuple_lend();
        a.extend_lender_one(x);
        b.extend_lender_one(y);
    }
    Ok((a, b))
}
