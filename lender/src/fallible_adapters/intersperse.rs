use core::{fmt, ops::ControlFlow};

use crate::{
    FallibleLend, FallibleLender, FallibleLending, FalliblePeekable, FusedFallibleLender,
    try_trait_v2::Try,
};

// Clone is not implemented because the inner Peekable is not Clone.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Intersperse<'this, L>
where
    for<'all> FallibleLend<'all, L>: Clone,
    L: FallibleLender,
{
    lender: FalliblePeekable<'this, L>,
    separator: FallibleLend<'this, L>,
    needs_sep: bool,
}

impl<'this, L> Intersperse<'this, L>
where
    for<'all> FallibleLend<'all, L>: Clone,
    L: FallibleLender,
{
    #[inline(always)]
    pub(crate) fn new(lender: L, separator: FallibleLend<'this, L>) -> Self {
        Self {
            lender: lender.peekable(),
            separator,
            needs_sep: false,
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender.into_inner()
    }

    /// Returns the inner lender and the separator value.
    #[inline(always)]
    pub fn into_parts(self) -> (L, FallibleLend<'this, L>) {
        (self.lender.into_inner(), self.separator)
    }
}

impl<L: fmt::Debug> fmt::Debug for Intersperse<'_, L>
where
    for<'all> FallibleLend<'all, L>: Clone + fmt::Debug,
    L: FallibleLender,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Intersperse")
            .field("lender", &self.lender)
            .field("separator", &self.separator)
            .field("needs_sep", &self.needs_sep)
            .finish()
    }
}

impl<'lend, L> FallibleLending<'lend> for Intersperse<'_, L>
where
    for<'all> FallibleLend<'all, L>: Clone,
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'this, L> FallibleLender for Intersperse<'this, L>
where
    for<'all> FallibleLend<'all, L>: Clone,
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.needs_sep && self.lender.peek()?.is_some() {
            self.needs_sep = false;
            Ok(Some(
                // SAFETY: 'this: 'lend
                unsafe {
                    core::mem::transmute::<FallibleLend<'this, Self>, FallibleLend<'_, Self>>(
                        self.separator.clone(),
                    )
                },
            ))
        } else {
            self.needs_sep = true;
            self.lender.next()
        }
    }

    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let mut acc = init;
        if !self.needs_sep {
            match self.lender.next()? {
                None => return Ok(R::from_output(acc)),
                Some(x) => {
                    acc = match f(acc, x)?.branch() {
                        ControlFlow::Continue(b) => b,
                        ControlFlow::Break(residual) => return Ok(R::from_residual(residual)),
                    };
                }
            }
        }
        let separator = &self.separator;
        self.lender.try_fold(acc, move |acc, x| {
            let acc = match f(acc, separator.clone())?.branch() {
                ControlFlow::Continue(b) => b,
                ControlFlow::Break(residual) => return Ok(R::from_residual(residual)),
            };
            f(acc, x)
        })
    }

    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        if !self.needs_sep {
            if let Some(x) = self.lender.next()? {
                acc = f(acc, x)?;
            } else {
                return Ok(acc);
            }
        }
        self.lender.fold(acc, |mut acc, x| {
            acc = f(acc, self.separator.clone())?;
            acc = f(acc, x)?;
            Ok(acc)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        intersperse_size_hint(&self.lender, self.needs_sep)
    }
}

// Clone is not implemented because the inner Peekable is not Clone.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct IntersperseWith<'this, L, G>
where
    L: FallibleLender,
{
    separator: G,
    lender: FalliblePeekable<'this, L>,
    needs_sep: bool,
}

impl<'this, L, G> IntersperseWith<'this, L, G>
where
    L: FallibleLender,
    G: FnMut() -> Result<FallibleLend<'this, L>, L::Error>,
{
    #[inline(always)]
    pub(crate) fn new(lender: L, separator: G) -> Self {
        Self {
            lender: FalliblePeekable::new(lender),
            separator,
            needs_sep: false,
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender.into_inner()
    }

    /// Returns the inner lender and the separator function.
    #[inline(always)]
    pub fn into_parts(self) -> (L, G) {
        (self.lender.into_inner(), self.separator)
    }
}

impl<L: fmt::Debug, G: fmt::Debug> fmt::Debug for IntersperseWith<'_, L, G>
where
    L: FallibleLender,
    for<'all> FallibleLend<'all, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntersperseWith")
            .field("lender", &self.lender)
            .field("separator", &self.separator)
            .field("needs_sep", &self.needs_sep)
            .finish()
    }
}

impl<'lend, 'this, L, G> FallibleLending<'lend> for IntersperseWith<'this, L, G>
where
    L: FallibleLender,
    G: FnMut() -> Result<FallibleLend<'this, L>, L::Error>,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'this, L, G> FallibleLender for IntersperseWith<'this, L, G>
where
    L: FallibleLender,
    G: FnMut() -> Result<FallibleLend<'this, L>, L::Error>,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.needs_sep && self.lender.peek()?.is_some() {
            self.needs_sep = false;
            let separator = (self.separator)()?;
            Ok(Some(
                // SAFETY: 'this: 'lend
                unsafe {
                    core::mem::transmute::<FallibleLend<'this, L>, FallibleLend<'_, L>>(separator)
                },
            ))
        } else {
            self.needs_sep = true;
            self.lender.next()
        }
    }

    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let mut acc = init;
        if !self.needs_sep {
            match self.lender.next()? {
                None => return Ok(R::from_output(acc)),
                Some(x) => {
                    acc = match f(acc, x)?.branch() {
                        ControlFlow::Continue(b) => b,
                        ControlFlow::Break(residual) => return Ok(R::from_residual(residual)),
                    };
                }
            }
        }
        let separator = &mut self.separator;
        self.lender.try_fold(acc, move |acc, x| {
            let acc = match f(acc, (separator)()?)?.branch() {
                ControlFlow::Continue(b) => b,
                ControlFlow::Break(residual) => return Ok(R::from_residual(residual)),
            };
            f(acc, x)
        })
    }

    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        if !self.needs_sep {
            if let Some(x) = self.lender.next()? {
                acc = f(acc, x)?;
            } else {
                return Ok(acc);
            }
        }
        self.lender.fold(acc, |mut acc, x| {
            acc = f(acc, (self.separator)()?)?;
            acc = f(acc, x)?;
            Ok(acc)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        intersperse_size_hint(&self.lender, self.needs_sep)
    }
}

fn intersperse_size_hint<L>(lender: &L, needs_sep: bool) -> (usize, Option<usize>)
where
    L: FallibleLender,
{
    let (lo, hi) = lender.size_hint();
    let next_is_elem = !needs_sep;
    (
        lo.saturating_sub(next_is_elem as usize).saturating_add(lo),
        hi.and_then(|hi| hi.saturating_sub(next_is_elem as usize).checked_add(hi)),
    )
}

impl<'this, L> FusedFallibleLender for Intersperse<'this, L>
where
    for<'all> FallibleLend<'all, L>: Clone,
    L: FusedFallibleLender,
{
}

impl<'this, L, G> FusedFallibleLender for IntersperseWith<'this, L, G>
where
    L: FusedFallibleLender,
    G: FnMut() -> Result<FallibleLend<'this, L>, L::Error>,
{
}
