use aliasable::boxed::AliasableBox;
use core::{fmt, ops::ControlFlow};
use maybe_dangling::MaybeDangling;

use crate::{
    DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending,
    try_trait_v2::{FromResidual, Try},
};

/// A lender with a [`peek()`](Peekable::peek) method that returns an optional
/// reference to the next element.
///
/// This `struct` is created by the [`peekable()`](crate::Lender::peekable) method on [`Lender`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Peekable<'this, L>
where
    L: Lender,
{
    // MaybeDangling wraps the peeked value to indicate it may reference data
    // from the lender. AliasableBox eliminates noalias retagging that would
    // invalidate the peeked reference when the struct is moved.
    // Field order ensures lender drops last.
    //
    // See https://github.com/WanderLanz/Lender/issues/34
    peeked: MaybeDangling<Option<Option<Lend<'this, L>>>>,
    lender: AliasableBox<L>,
}
impl<'this, L> Peekable<'this, L>
where
    L: Lender,
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

    /// Returns a reference to the next element without advancing the lender.
    ///
    /// Like [`next`](Lender::next), if there is a next value, it is borrowed from the
    /// underlying lender and cached. Calling `peek()` multiple times without advancing
    /// the lender returns the same cached element.
    ///
    /// # Examples
    ///
    /// ```
    /// use lender::prelude::*;
    ///
    /// let mut lender = lender::from_iter([1, 2, 3].iter().copied()).peekable();
    ///
    /// assert_eq!(lender.peek(), Some(&1));
    /// assert_eq!(lender.peek(), Some(&1)); // Doesn't advance
    /// assert_eq!(lender.next(), Some(1));
    /// assert_eq!(lender.peek(), Some(&2));
    /// ```
    pub fn peek(&mut self) -> Option<&'_ Lend<'_, L>> {
        let lender = &mut self.lender;
        // SAFETY: Two transmutes are used here:
        // 1. Inner: `Option<Lend<'_, L>>` to `Option<Lend<'this, L>>` - extends the
        //    lend's lifetime to store it in `self.peeked`. Safe because the lender
        //    is boxed (stable address) and only one lend is alive at a time.
        // 2. Outer: `Option<&'_ Lend<'this, L>>` to `Option<&'_ Lend<'_, L>>` - ties
        //    the lend's lifetime to the borrow of `self`, preventing it from escaping.
        //    Safe because `L::Lend` is covariant in its lifetime (required by Lender).
        unsafe {
            core::mem::transmute::<Option<&'_ Lend<'this, L>>, Option<&'_ Lend<'_, L>>>(
                self.peeked
                    .get_or_insert_with(|| {
                        core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(
                            lender.next(),
                        )
                    })
                    .as_ref(),
            )
        }
    }

    /// Returns a mutable reference to the next element without advancing the lender.
    ///
    /// Like [`peek`](Self::peek), if there is a next value, it is borrowed from the
    /// underlying lender and cached. The returned mutable reference allows modifying
    /// the peeked value.
    ///
    /// # Examples
    ///
    /// ```
    /// use lender::prelude::*;
    ///
    /// let mut lender = lender::from_iter([1, 2, 3].iter().copied()).peekable();
    ///
    /// if let Some(p) = lender.peek_mut() {
    ///     *p = 10;
    /// }
    /// assert_eq!(lender.next(), Some(10));
    /// assert_eq!(lender.next(), Some(2));
    /// ```
    pub fn peek_mut(&mut self) -> Option<&'_ mut Lend<'this, L>> {
        let lender = &mut self.lender;
        self.peeked
            .get_or_insert_with(|| {
                // SAFETY: The lend is manually guaranteed to be the only one alive
                unsafe {
                    core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(
                        lender.next(),
                    )
                }
            })
            .as_mut()
    }

    /// Consumes and returns the next element if the given predicate is true.
    ///
    /// If `f(&next_element)` returns `true`, consumes and returns the next element.
    /// Otherwise, returns `None` and the element remains peeked.
    ///
    /// # Examples
    ///
    /// ```
    /// use lender::prelude::*;
    ///
    /// let mut lender = lender::from_iter([1, 2, 3].iter().copied()).peekable();
    ///
    /// // Consume 1 since it's odd
    /// assert_eq!(lender.next_if(|&x| x % 2 == 1), Some(1));
    /// // Don't consume 2 since it's not odd
    /// assert_eq!(lender.next_if(|&x| x % 2 == 1), None);
    /// // 2 is still there
    /// assert_eq!(lender.next(), Some(2));
    /// ```
    pub fn next_if<F>(&mut self, f: F) -> Option<Lend<'_, L>>
    where
        F: FnOnce(&Lend<'_, L>) -> bool,
    {
        // Get the next value by inlining the logic of next() to avoid borrow conflicts
        let v = match self.peeked.take() {
            // SAFETY: The lend is manually guaranteed to be the only one alive
            Some(peeked) => unsafe {
                core::mem::transmute::<Option<Lend<'this, L>>, Option<Lend<'_, L>>>(peeked)
            },
            None => self.lender.next(),
        };
        match v {
            Some(v) if f(&v) => Some(v),
            v => {
                // SAFETY: The lend is manually guaranteed to be the only one alive
                *self.peeked = Some(unsafe {
                    core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(v)
                });
                None
            }
        }
    }

    /// Consumes and returns the next element if it equals the given value.
    ///
    /// If the next element equals `t`, consumes and returns it. Otherwise,
    /// returns `None` and the element remains peeked.
    ///
    /// # Examples
    ///
    /// ```
    /// use lender::prelude::*;
    ///
    /// let mut lender = lender::from_iter([1, 2, 3].iter().copied()).peekable();
    ///
    /// // Consume 1 since it equals 1
    /// assert_eq!(lender.next_if_eq(&1), Some(1));
    /// // Don't consume 2 since it doesn't equal 1
    /// assert_eq!(lender.next_if_eq(&1), None);
    /// // 2 is still there
    /// assert_eq!(lender.next(), Some(2));
    /// ```
    pub fn next_if_eq<'a, T>(&'a mut self, t: &T) -> Option<Lend<'a, L>>
    where
        T: for<'all> PartialEq<Lend<'all, L>>,
    {
        self.next_if(|v| t == v)
    }
}

// Clone is not implemented for Peekable because the peeked value borrows from
// the lender's AliasableBox allocation; a clone would need its own allocation,
// leaving the cloned peeked value dangling.

impl<'this, L: fmt::Debug> fmt::Debug for Peekable<'this, L>
where
    L: Lender + fmt::Debug,
    Lend<'this, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Peekable")
            .field("lender", &self.lender)
            .field("peeked", &self.peeked)
            .finish()
    }
}

impl<'lend, L> Lending<'lend> for Peekable<'_, L>
where
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}

impl<'this, L> Lender for Peekable<'this, L>
where
    L: Lender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        match self.peeked.take() {
            // SAFETY: The lend is manually guaranteed to be the only one alive
            Some(peeked) => unsafe {
                core::mem::transmute::<Option<Lend<'this, Self>>, Option<Lend<'_, Self>>>(peeked)
            },
            None => self.lender.next(),
        }
    }

    #[inline]
    fn count(mut self) -> usize {
        let lender = *AliasableBox::into_unique(self.lender);
        match self.peeked.take() {
            Some(None) => 0,
            Some(Some(_)) => 1 + lender.count(),
            None => lender.count(),
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        match self.peeked.take() {
            Some(None) => None,
            // SAFETY: The lend is manually guaranteed to be the only one alive
            Some(v @ Some(_)) if n == 0 => unsafe {
                core::mem::transmute::<Option<Lend<'this, Self>>, Option<Lend<'_, Self>>>(v)
            },
            Some(Some(_)) => self.lender.nth(n - 1),
            None => self.lender.nth(n),
        }
    }

    #[inline]
    fn last<'a>(&'a mut self) -> Option<Lend<'a, Self>>
    where
        Self: Sized,
    {
        let peek_opt = match self.peeked.take() {
            Some(None) => return None,
            // SAFETY: 'this: 'call
            Some(v) => unsafe {
                core::mem::transmute::<Option<Lend<'this, Self>>, Option<Lend<'a, Self>>>(v)
            },
            None => None,
        };
        self.lender.last().or(peek_opt)
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
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let acc = match self.peeked.take() {
            Some(None) => return Try::from_output(init),
            Some(Some(v)) => match f(init, v).branch() {
                ControlFlow::Break(b) => return FromResidual::from_residual(b),
                ControlFlow::Continue(a) => a,
            },
            None => init,
        };
        self.lender.try_fold(acc, f)
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        match self.peeked.take() {
            Some(None) => init,
            Some(Some(v)) => {
                // Manual loop instead of lender.fold() to avoid
                // consuming the lender before v is used: v borrows
                // from the AliasableBox allocation, which must stay
                // alive until f(acc, v) completes.
                let mut acc = f(init, v);
                while let Some(x) = self.lender.next() {
                    acc = f(acc, x);
                }
                acc
            }
            None => {
                let lender = *AliasableBox::into_unique(self.lender);
                lender.fold(init, f)
            }
        }
    }
}

impl<'this, L: DoubleEndedLender> DoubleEndedLender for Peekable<'this, L> {
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        match self.peeked.as_mut() {
            // SAFETY: The lend is manually guaranteed to be the only one alive
            Some(v @ Some(_)) => self.lender.next_back().or_else(|| unsafe {
                core::mem::transmute::<Option<Lend<'this, Self>>, Option<Lend<'_, Self>>>(v.take())
            }),
            Some(None) => None,
            None => self.lender.next_back(),
        }
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        match self.peeked.take() {
            None => self.lender.try_rfold(init, f),
            Some(None) => Try::from_output(init),
            Some(Some(v)) => match self.lender.try_rfold(init, &mut f).branch() {
                ControlFlow::Continue(acc) => f(acc, v),
                ControlFlow::Break(r) => {
                    *self.peeked = Some(Some(v));
                    FromResidual::from_residual(r)
                }
            },
        }
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        match self.peeked.take() {
            None => {
                let lender = *AliasableBox::into_unique(self.lender);
                lender.rfold(init, f)
            }
            Some(None) => init,
            Some(Some(v)) => {
                // Manual loop instead of lender.rfold() to avoid
                // consuming the lender before v is used: v borrows
                // from the AliasableBox allocation, which must stay
                // alive until f(acc, v) completes.
                let mut acc = init;
                while let Some(x) = self.lender.next_back() {
                    acc = f(acc, x);
                }
                f(acc, v)
            }
        }
    }
}

impl<L: ExactSizeLender> ExactSizeLender for Peekable<'_, L> {}

impl<L: FusedLender> FusedLender for Peekable<'_, L> {}

#[cfg(test)]
mod test {
    use super::*;

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
    fn test_peekable() {
        let lender = ArrayLender {
            array: [-1, 1, 2, 3],
        };
        let mut peekable = lender.peekable();
        assert_eq!(**peekable.peek().unwrap(), -1);
        assert_eq!(
            peekable.peeked.unwrap().unwrap() as *const _,
            &peekable.lender.array[0] as *const _
        );
        moved_peekable(peekable);
    }

    fn moved_peekable(peekable: Peekable<ArrayLender>) {
        let peeked = peekable.peeked.unwrap().unwrap() as *const _;
        let array = &peekable.lender.array[0] as *const _;
        assert_eq!(
            peeked, array,
            "Peeked element pointer should point to the first element of the array"
        );
    }
}
