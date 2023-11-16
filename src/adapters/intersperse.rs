use core::fmt;

use crate::{Lend, Lender, Lending, Peekable};

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Intersperse<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Clone,
    L: Lender,
{
    lender: Peekable<'this, L>,
    separator: <L as Lending<'this>>::Lend,
    needs_sep: bool,
}
impl<'this, L> Intersperse<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Clone,
    L: Lender,
{
    pub(crate) fn new(lender: L, separator: <L as Lending<'this>>::Lend) -> Self {
        Self { lender: lender.peekable(), separator, needs_sep: false }
    }
}
impl<'this, L: fmt::Debug> fmt::Debug for Intersperse<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Clone + fmt::Debug,
    L: Lender,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Intersperse")
            .field("lender", &self.lender)
            .field("separator", &self.separator)
            .field("needs_sep", &self.needs_sep)
            .finish()
    }
}
impl<'lend, 'this, L> Lending<'lend> for Intersperse<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Clone,
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}
impl<'this, L> Lender for Intersperse<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Clone,
    L: Lender,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if self.needs_sep && self.lender.peek().is_some() {
            self.needs_sep = false;
            // SAFETY: 'this: 'lend
            Some(unsafe {
                core::mem::transmute::<<Self as Lending<'this>>::Lend, <Self as Lending<'_>>::Lend>(self.separator.clone())
            })
        } else {
            self.needs_sep = true;
            self.lender.next()
        }
    }
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        let mut acc = init;
        if !self.needs_sep {
            if let Some(x) = self.lender.next() {
                acc = f(acc, x);
            } else {
                return acc;
            }
        }
        self.lender.fold(acc, |mut acc, x| {
            acc = f(acc, self.separator.clone());
            acc = f(acc, x);
            acc
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        intersperse_size_hint(&self.lender, self.needs_sep)
    }
}

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct IntersperseWith<'this, L, G>
where
    L: Lender,
{
    separator: G,
    lender: Peekable<'this, L>,
    needs_sep: bool,
}
impl<'this, L, G> IntersperseWith<'this, L, G>
where
    L: Lender,
    G: FnMut() -> <L as Lending<'this>>::Lend,
{
    pub(crate) fn new(lender: L, seperator: G) -> Self {
        Self { lender: Peekable::new(lender), separator: seperator, needs_sep: false }
    }
}
impl<'this, L: fmt::Debug, G: fmt::Debug> fmt::Debug for IntersperseWith<'this, L, G>
where
    L: Lender,
    for<'all> <L as Lending<'all>>::Lend: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntersperseWith")
            .field("lender", &self.lender)
            .field("separator", &self.separator)
            .field("needs_sep", &self.needs_sep)
            .finish()
    }
}
impl<'lend, 'this, L, G> Lending<'lend> for IntersperseWith<'this, L, G>
where
    L: Lender,
    G: FnMut() -> <L as Lending<'this>>::Lend,
{
    type Lend = Lend<'lend, L>;
}
impl<'this, L, G> Lender for IntersperseWith<'this, L, G>
where
    L: Lender,
    G: FnMut() -> <L as Lending<'this>>::Lend,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if self.needs_sep && self.lender.peek().is_some() {
            self.needs_sep = false;
            // SAFETY: 'this: 'lend
            Some(unsafe {
                core::mem::transmute::<<L as Lending<'this>>::Lend, <L as Lending<'_>>::Lend>((self.separator)())
            })
        } else {
            self.needs_sep = true;
            self.lender.next()
        }
    }
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        let mut acc = init;
        if !self.needs_sep {
            if let Some(x) = self.lender.next() {
                acc = f(acc, x);
            } else {
                return acc;
            }
        }
        self.lender.fold(acc, |mut acc, x| {
            acc = f(acc, (self.separator)());
            acc = f(acc, x);
            acc
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        intersperse_size_hint(&self.lender, self.needs_sep)
    }
}

fn intersperse_size_hint<L>(lender: &L, needs_sep: bool) -> (usize, Option<usize>)
where
    L: Lender,
{
    let (lo, hi) = lender.size_hint();
    let next_is_elem = !needs_sep;
    (
        lo.saturating_sub(next_is_elem as usize).saturating_add(lo),
        hi.and_then(|hi| hi.saturating_sub(next_is_elem as usize).checked_add(hi)),
    )
}
