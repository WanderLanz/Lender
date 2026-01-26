use core::fmt;

use crate::{FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender, Lending};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct TakeWhile<L, P> {
    lender: L,
    flag: bool,
    predicate: P,
}
impl<L, P> TakeWhile<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> TakeWhile<L, P> {
        TakeWhile { lender, flag: false, predicate }
    }
    pub fn into_inner(self) -> L {
        self.lender
    }
    pub fn into_parts(self) -> (L, P) {
        (self.lender, self.predicate)
    }
}
impl<L: fmt::Debug, P> fmt::Debug for TakeWhile<L, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TakeWhile").field("lender", &self.lender).field("flag", &self.flag).finish()
    }
}
impl<'lend, L, P> Lending<'lend> for TakeWhile<L, P>
where
    P: FnMut(&Lend<'lend, L>) -> bool,
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}
impl<L, P> Lender for TakeWhile<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: Lender,
{
    crate::inherit_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if !self.flag {
            let x = self.lender.next()?;
            if (self.predicate)(&x) {
                return Some(x);
            }
            self.flag = true;
        }
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.flag {
            (0, Some(0))
        } else {
            let (_, upper) = self.lender.size_hint();
            (0, upper)
        }
    }
}
impl<L, P> FusedLender for TakeWhile<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: Lender,
{
}

impl<'lend, L, P> FallibleLending<'lend> for TakeWhile<L, P>
where
    P: FnMut(&FallibleLend<'lend, L>) -> Result<bool, L::Error>,
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}
impl<L, P> FallibleLender for TakeWhile<L, P>
where
    P: FnMut(&FallibleLend<'_, L>) -> Result<bool, L::Error>,
    L: FallibleLender,
{
    type Error = L::Error;
    crate::inherit_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if !self.flag {
            let x = match self.lender.next()? {
                Some(x) => x,
                None => return Ok(None),
            };
            if (self.predicate)(&x)? {
                return Ok(Some(x));
            }
            self.flag = true;
        }
        Ok(None)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.flag {
            (0, Some(0))
        } else {
            let (_, upper) = self.lender.size_hint();
            (0, upper)
        }
    }
}
impl<L, P> FusedFallibleLender for TakeWhile<L, P>
where
    P: FnMut(&FallibleLend<'_, L>) -> Result<bool, L::Error>,
    L: FallibleLender,
{
}
