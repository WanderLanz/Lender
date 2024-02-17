use core::fmt;

use crate::{HasNext, Lend, Lender, Lending};

#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Intersperse<'this, L>
where
    for<'all> Lend<'all, L>: Clone,
    L: Lender + HasNext,
{
    lender: L,
    separator: Lend<'this, L>,
    needs_sep: bool,
}
impl<'this, L> Intersperse<'this, L>
where
    for<'all> Lend<'all, L>: Clone,
    L: Lender + HasNext,
{
    pub(crate) fn new(lender: L, separator: Lend<'this, L>) -> Self { Self { lender, separator, needs_sep: false } }
}
impl<'this, L: fmt::Debug> fmt::Debug for Intersperse<'this, L>
where
    for<'all> Lend<'all, L>: Clone + fmt::Debug,
    L: Lender + HasNext,
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
    for<'all> Lend<'all, L>: Clone,
    L: Lender + HasNext,
{
    type Lend = Lend<'lend, L>;
}
impl<'this, L> Lender for Intersperse<'this, L>
where
    for<'all> Lend<'all, L>: Clone,
    L: Lender + HasNext,
{
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.needs_sep && self.lender.has_next() {
            self.needs_sep = false;
            // SAFETY: 'this: 'lend
            Some(unsafe { core::mem::transmute::<Lend<'this, Self>, Lend<'_, Self>>(self.separator.clone()) })
        } else {
            self.needs_sep = true;
            self.lender.next()
        }
    }
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
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
    fn size_hint(&self) -> (usize, Option<usize>) { intersperse_size_hint(&self.lender, self.needs_sep) }
}

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct IntersperseWith<'this, L, G> {
    separator: G,
    lender: L,
    needs_sep: bool,
    _phantom: core::marker::PhantomData<&'this ()>,
}
impl<'this, L: 'this, G> IntersperseWith<'this, L, G>
where
    L: Lender + HasNext,
    G: FnMut() -> Lend<'this, L>,
{
    pub(crate) fn new(lender: L, seperator: G) -> Self {
        Self { lender, separator: seperator, needs_sep: false, _phantom: core::marker::PhantomData }
    }
}
impl<'this, L: fmt::Debug, G: fmt::Debug> fmt::Debug for IntersperseWith<'this, L, G>
where
    L: Lender + HasNext,
    for<'all> Lend<'all, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntersperseWith")
            .field("lender", &self.lender)
            .field("separator", &self.separator)
            .field("needs_sep", &self.needs_sep)
            .finish()
    }
}
impl<'lend, 'this, L: 'this, G> Lending<'lend> for IntersperseWith<'this, L, G>
where
    L: Lender + HasNext,
    G: FnMut() -> Lend<'this, L>,
{
    type Lend = Lend<'lend, L>;
}
impl<'this, L: 'this, G> Lender for IntersperseWith<'this, L, G>
where
    L: Lender + HasNext,
    G: FnMut() -> Lend<'this, L>,
{
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.needs_sep && self.lender.has_next() {
            self.needs_sep = false;
            // SAFETY: 'this: 'lend
            Some(unsafe { core::mem::transmute::<Lend<'this, L>, Lend<'_, L>>((self.separator)()) })
        } else {
            self.needs_sep = true;
            self.lender.next()
        }
    }
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
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
    fn size_hint(&self) -> (usize, Option<usize>) { intersperse_size_hint(&self.lender, self.needs_sep) }
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
