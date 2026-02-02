use aliasable::boxed::AliasableBox;
use core::{fmt, ops::ControlFlow};
use maybe_dangling::MaybeDangling;

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender,
    try_trait_v2::{FromResidual, Try},
};

/// A fallible lender with a [`peek()`](Peekable::peek) method that returns an optional
/// reference to the next element.
///
/// This `struct` is created by the [`peekable()`](crate::FallibleLender::peekable) method on
/// [`FallibleLender`]. See its documentation for more.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Peekable<'this, L>
where
    L: FallibleLender,
{
    // MaybeDangling wraps the peeked value to indicate it may reference data
    // from the lender. AliasableBox eliminates noalias retagging that would
    // invalidate the peeked reference when the struct is moved.
    // Field order ensures lender drops last.
    peeked: MaybeDangling<Option<Option<FallibleLend<'this, L>>>>,
    lender: AliasableBox<L>,
}

impl<'this, L> Peekable<'this, L>
where
    L: FallibleLender,
{
    #[inline(always)]
    pub(crate) fn new(lender: L) -> Peekable<'this, L> {
        Peekable {
            peeked: MaybeDangling::new(None),
            lender: AliasableBox::from_unique(alloc::boxed::Box::new(lender)),
        }
    }

    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        *AliasableBox::into_unique(self.lender)
    }

    pub fn peek(&mut self) -> Result<Option<&'_ FallibleLend<'_, L>>, L::Error> {
        let lender = &mut self.lender;
        if self.peeked.is_none() {
            // SAFETY: Extends the lend's lifetime to store it in `self.peeked`.
            // Safe because the lender is boxed (stable address) and only one lend
            // is alive at a time.
            *self.peeked = Some(unsafe {
                core::mem::transmute::<Option<FallibleLend<'_, L>>, Option<FallibleLend<'this, L>>>(
                    lender.next()?,
                )
            });
        }
        // SAFETY: Ties the lend's lifetime to the borrow of `self`, preventing it
        // from escaping. Safe because `L::Lend` is covariant in its lifetime
        // (required by FallibleLender). The `unwrap_unchecked` is safe because
        // `self.peeked` was set to `Some` above if it was `None`.
        Ok(unsafe {
            core::mem::transmute::<
                Option<&'_ FallibleLend<'this, L>>,
                Option<&'_ FallibleLend<'_, L>>,
            >(self.peeked.as_mut().unwrap_unchecked().as_ref())
        })
    }

    pub fn peek_mut(&mut self) -> Result<Option<&'_ mut FallibleLend<'this, L>>, L::Error> {
        let lender = &mut self.lender;
        if self.peeked.is_none() {
            *self.peeked = Some(
                // SAFETY: The lend is manually guaranteed to be the only one alive
                unsafe {
                    core::mem::transmute::<
                        Option<FallibleLend<'_, L>>,
                        Option<FallibleLend<'this, L>>,
                    >(lender.next()?)
                },
            );
        }
        Ok(
            // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
            // variant in the code above.
            unsafe { self.peeked.as_mut().unwrap_unchecked().as_mut() },
        )
    }

    pub fn next_if<F>(&mut self, f: F) -> Result<Option<FallibleLend<'_, L>>, L::Error>
    where
        F: FnOnce(&FallibleLend<'_, L>) -> bool,
    {
        // Get the next value by inlining the logic of next() to avoid borrow conflicts
        let v = match self.peeked.take() {
            // SAFETY: The lend is manually guaranteed to be the only one alive
            Some(peeked) => unsafe {
                core::mem::transmute::<Option<FallibleLend<'this, L>>, Option<FallibleLend<'_, L>>>(
                    peeked,
                )
            },
            None => self.lender.next()?,
        };
        match v {
            Some(v) if f(&v) => Ok(Some(v)),
            v => {
                // SAFETY: The lend is manually guaranteed to be the only one alive
                *self.peeked = Some(unsafe {
                    core::mem::transmute::<
                        Option<FallibleLend<'_, L>>,
                        Option<FallibleLend<'this, L>>,
                    >(v)
                });
                Ok(None)
            }
        }
    }

    pub fn next_if_eq<'a, T>(&'a mut self, t: &T) -> Result<Option<FallibleLend<'a, L>>, L::Error>
    where
        T: for<'all> PartialEq<FallibleLend<'all, L>>,
    {
        self.next_if(|v| t == v)
    }
}

// Clone is not implemented for Peekable because the peeked value borrows from
// the lender's AliasableBox allocation; a clone would need its own allocation,
// leaving the cloned peeked value dangling.

impl<'this, L: fmt::Debug> fmt::Debug for Peekable<'this, L>
where
    L: FallibleLender + fmt::Debug,
    FallibleLend<'this, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Peekable")
            .field("lender", &self.lender)
            .field("peeked", &self.peeked)
            .finish()
    }
}

impl<'lend, L> FallibleLending<'lend> for Peekable<'_, L>
where
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'this, L> FallibleLender for Peekable<'this, L>
where
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.peeked.take() {
            Some(peeked) => Ok(
                // SAFETY: The lend is manually guaranteed to be the only one alive
                unsafe {
                    core::mem::transmute::<
                        Option<FallibleLend<'this, Self>>,
                        Option<FallibleLend<'_, Self>>,
                    >(peeked)
                },
            ),
            None => self.lender.next(),
        }
    }

    #[inline]
    fn count(mut self) -> Result<usize, Self::Error> {
        let lender = *AliasableBox::into_unique(self.lender);
        match self.peeked.take() {
            Some(None) => Ok(0),
            Some(Some(_)) => Ok(1 + lender.count()?),
            None => lender.count(),
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.peeked.take() {
            Some(None) => Ok(None),
            Some(v @ Some(_)) if n == 0 => Ok(unsafe {
                // SAFETY: The lend is manually guaranteed to be the only one alive
                core::mem::transmute::<
                    Option<FallibleLend<'this, Self>>,
                    Option<FallibleLend<'_, Self>>,
                >(v)
            }),
            Some(Some(_)) => self.lender.nth(n - 1),
            None => self.lender.nth(n),
        }
    }

    #[inline]
    fn last<'a>(&'a mut self) -> Result<Option<FallibleLend<'a, Self>>, Self::Error>
    where
        Self: Sized,
    {
        let peek_opt = match self.peeked.take() {
            Some(None) => return Ok(None),
            Some(v) =>
            // SAFETY: 'this: 'call
            unsafe {
                core::mem::transmute::<
                    Option<FallibleLend<'this, Self>>,
                    Option<FallibleLend<'a, Self>>,
                >(v)
            },
            None => None,
        };
        Ok(self.lender.last()?.or(peek_opt))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let peek_len = match *self.peeked {
            Some(None) => return (0, Some(0)),
            Some(Some(_)) => 1,
            None => 0,
        };
        let (l, r) = self.lender.size_hint();
        (
            l.saturating_add(peek_len),
            r.and_then(|r| r.checked_add(peek_len)),
        )
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let acc = match self.peeked.take() {
            Some(None) => return Ok(Try::from_output(init)),
            Some(Some(v)) => match f(init, v)?.branch() {
                ControlFlow::Break(b) => return Ok(FromResidual::from_residual(b)),
                ControlFlow::Continue(a) => a,
            },
            None => init,
        };
        self.lender.try_fold(acc, f)
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let lender = *AliasableBox::into_unique(self.lender);
        let acc = match self.peeked.take() {
            Some(None) => return Ok(init),
            Some(Some(v)) => f(init, v)?,
            None => init,
        };
        lender.fold(acc, f)
    }
}

impl<'this, L: DoubleEndedFallibleLender> DoubleEndedFallibleLender for Peekable<'this, L> {
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.peeked.as_mut() {
            Some(v @ Some(_)) => match self.lender.next_back()? {
                Some(next) => Ok(Some(next)),
                None => Ok(
                    // SAFETY: The lend is manually guaranteed to be the only one alive
                    unsafe {
                        core::mem::transmute::<
                            Option<FallibleLend<'this, Self>>,
                            Option<FallibleLend<'_, Self>>,
                        >(v.take())
                    },
                ),
            },
            Some(None) => Ok(None),
            None => self.lender.next_back(),
        }
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        match self.peeked.take() {
            None => self.lender.try_rfold(init, f),
            Some(None) => Ok(Try::from_output(init)),
            Some(Some(v)) => match self.lender.try_rfold(init, &mut f)?.branch() {
                ControlFlow::Continue(acc) => f(acc, v),
                ControlFlow::Break(r) => {
                    *self.peeked = Some(Some(v));
                    Ok(FromResidual::from_residual(r))
                }
            },
        }
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        match self.peeked.take() {
            None => {
                let lender = *AliasableBox::into_unique(self.lender);
                lender.rfold(init, f)
            }
            Some(None) => Ok(init),
            Some(Some(v)) => {
                // Manual loop instead of lender.rfold() to avoid
                // consuming the lender before v is used: v borrows
                // from the AliasableBox allocation, which must stay
                // alive until f(acc, v) completes.
                let mut acc = init;
                while let Some(x) = self.lender.next_back()? {
                    acc = f(acc, x)?;
                }
                f(acc, v)
            }
        }
    }
}

impl<'this, L> ExactSizeFallibleLender for Peekable<'this, L> where L: ExactSizeFallibleLender {}

impl<'this, L> FusedFallibleLender for Peekable<'this, L> where L: FusedFallibleLender {}

#[cfg(test)]
mod test {
    use core::convert::Infallible;

    use super::*;
    use crate::{IntoFallible, Lend, Lender, Lending};

    struct ArrayLender {
        array: [i32; 4],
    }

    impl<'lend> Lending<'lend> for ArrayLender {
        type Lend = &'lend i32;
    }

    impl Lender for ArrayLender {
        crate::check_covariance!();
        fn next(&mut self) -> Option<Lend<'_, Self>> {
            Some(&self.array[0])
        }
    }

    // This test will fail if Peekable stores L instead of Box<L>. In that case,
    // when Peekable<ArrayLender> is moved, the array inside ArrayLender is
    // moved, too, but Peekable.peeked will still contain a reference to the
    // previous location.
    #[test]
    fn test_peekable() -> Result<(), Infallible> {
        let lender = ArrayLender {
            array: [-1, 1, 2, 3],
        };
        let mut peekable = lender.into_fallible().peekable();
        assert_eq!(**peekable.peek()?.unwrap(), -1);
        assert_eq!(
            peekable.peeked.unwrap().unwrap() as *const _,
            &peekable.lender.lender.array[0] as *const _
        );
        moved_peekable(peekable);
        Ok(())
    }

    fn moved_peekable(peekable: Peekable<IntoFallible<Infallible, ArrayLender>>) {
        let peeked = peekable.peeked.unwrap().unwrap() as *const _;
        let array = &peekable.lender.lender.array[0] as *const _;
        assert_eq!(
            peeked, array,
            "Peeked element pointer should point to the first element of the array"
        );
    }
}
