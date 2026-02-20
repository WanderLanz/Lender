use core::fmt;

use crate::{FusedLender, Lend, Lender, Lending, Peekable};

/// A lender that inserts a separator between adjacent elements of the
/// underlying lender.
///
/// This `struct` is created by the
/// [`intersperse()`](crate::Lender::intersperse) method on
/// [`Lender`].
// Clone is not implemented because the inner Peekable is not Clone.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Intersperse<'this, L>
where
    for<'all> Lend<'all, L>: Clone,
    L: Lender,
{
    // Field order ensures lender drops last
    separator: Lend<'this, L>,
    needs_sep: bool,
    lender: Peekable<'this, L>,
}

impl<'this, L> Intersperse<'this, L>
where
    for<'all> Lend<'all, L>: Clone,
    L: Lender,
{
    #[inline]
    pub(crate) fn new(lender: L, separator: Lend<'this, L>) -> Self {
        crate::__check_lender_covariance::<L>();
        Self {
            separator,
            needs_sep: false,
            lender: lender.peekable(),
        }
    }

    /// Returns the inner lender.
    #[inline]
    pub fn into_inner(self) -> L {
        self.lender.into_inner()
    }

    /// Returns the inner lender and the separator value.
    #[inline]
    pub fn into_parts(self) -> (L, Lend<'this, L>) {
        (self.lender.into_inner(), self.separator)
    }
}

impl<L: fmt::Debug> fmt::Debug for Intersperse<'_, L>
where
    for<'all> Lend<'all, L>: Clone + fmt::Debug,
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

impl<'lend, L> Lending<'lend> for Intersperse<'_, L>
where
    for<'all> Lend<'all, L>: Clone,
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}

impl<'this, L> Lender for Intersperse<'this, L>
where
    for<'all> Lend<'all, L>: Clone,
    L: Lender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.needs_sep && self.lender.peek().is_some() {
            self.needs_sep = false;
            // SAFETY: 'this: 'lend
            Some(unsafe {
                core::mem::transmute::<Lend<'this, Self>, Lend<'_, Self>>(self.separator.clone())
            })
        } else {
            self.needs_sep = true;
            self.lender.next()
        }
    }

    #[inline]
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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        intersperse_size_hint(&self.lender, self.needs_sep)
    }
}

/// A lender that inserts an element computed by a closure between adjacent
/// elements of the underlying lender.
///
/// This `struct` is created by the
/// [`intersperse_with()`](crate::Lender::intersperse_with)
/// method on [`Lender`].
// Clone is not implemented because the inner Peekable is not Clone.
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
    G: FnMut() -> Lend<'this, L>,
{
    #[inline]
    pub(crate) fn new(lender: L, separator: G) -> Self {
        crate::__check_lender_covariance::<L>();
        Self {
            lender: Peekable::new(lender),
            separator,
            needs_sep: false,
        }
    }

    /// Returns the inner lender.
    #[inline]
    pub fn into_inner(self) -> L {
        self.lender.into_inner()
    }

    /// Returns the inner lender and the separator function.
    #[inline]
    pub fn into_parts(self) -> (L, G) {
        (self.lender.into_inner(), self.separator)
    }
}

impl<L: fmt::Debug, G> fmt::Debug for IntersperseWith<'_, L, G>
where
    L: Lender,
    for<'all> Lend<'all, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntersperseWith")
            .field("lender", &self.lender)
            .field("needs_sep", &self.needs_sep)
            .finish_non_exhaustive()
    }
}

impl<'lend, 'this, L, G> Lending<'lend> for IntersperseWith<'this, L, G>
where
    L: Lender,
    G: FnMut() -> Lend<'this, L>,
{
    type Lend = Lend<'lend, L>;
}

impl<'this, L, G> Lender for IntersperseWith<'this, L, G>
where
    L: Lender,
    G: FnMut() -> Lend<'this, L>,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.needs_sep && self.lender.peek().is_some() {
            self.needs_sep = false;
            // SAFETY: 'this: 'lend
            Some(unsafe { core::mem::transmute::<Lend<'this, L>, Lend<'_, L>>((self.separator)()) })
        } else {
            self.needs_sep = true;
            self.lender.next()
        }
    }

    #[inline]
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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        intersperse_size_hint(&self.lender, self.needs_sep)
    }
}

#[inline]
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

impl<'this, L> FusedLender for Intersperse<'this, L>
where
    for<'all> Lend<'all, L>: Clone,
    L: FusedLender,
{
}

impl<'this, L, G> FusedLender for IntersperseWith<'this, L, G>
where
    L: FusedLender,
    G: FnMut() -> Lend<'this, L>,
{
}
