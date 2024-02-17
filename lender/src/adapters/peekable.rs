use core::{fmt, marker::PhantomPinned, pin::Pin};

use stable_try_trait_v2::{try_, Try};

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, HasNext, Lend, Lender, Lending};

#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Peekable<'this, L>
where
    L: Lender,
{
    lender: L,
    peeked: Option<Option<Lend<'this, L>>>,
    _pin: PhantomPinned,
}
impl<'this, L> Peekable<'this, L>
where
    L: Lender,
{
    pub(crate) fn new(lender: L) -> Peekable<'this, L> { Peekable { lender, peeked: None, _pin: PhantomPinned } }
    pub fn peek(self: Pin<&mut Self>) -> Option<&'_ Lend<'this, L>> {
        unsafe {
            let this = self.get_unchecked_mut();
            let peeked = &mut this.peeked;
            if let None = peeked {
                *peeked = Some({
                    // SAFETY: The lend is manually guaranteed to be the only one alive
                    core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(this.lender.next())
                });
            }
            // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
            // variant in the code above.
            peeked.as_ref().unwrap_unchecked().as_ref()
        }
    }
    pub fn peek_mut(self: Pin<&mut Self>) -> Option<&'_ mut Lend<'this, L>> {
        unsafe {
            let this = self.get_unchecked_mut();
            let peeked = &mut this.peeked;
            if let None = peeked {
                *peeked = Some({
                    // SAFETY: The lend is manually guaranteed to be the only one alive
                    core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(this.lender.next())
                });
            }
            // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
            // variant in the code above.
            peeked.as_mut().unwrap_unchecked().as_mut()
        }
    }
    pub fn next_if<F>(self: Pin<&mut Self>, f: F) -> Option<Lend<'_, L>>
    where
        F: FnOnce(&Lend<'_, L>) -> bool,
    {
        // SAFETY: we aren't moving self
        let this = unsafe { self.get_unchecked_mut() };
        this.peeked = match this.peeked.take() {
            Some(Some(v)) => {
                if f(&v) {
                    // SAFETY: 'this: 'call
                    return Some(unsafe { core::mem::transmute::<Lend<'this, L>, Lend<'_, L>>(v) });
                }
                Some(Some(v))
            }
            None => match this.lender.next() {
                Some(v) if f(&v) => return Some(v),
                // SAFETY: The lend is manually guaranteed to be the only one alive and we pin to avoid some largening pitfalls
                v => Some(unsafe { core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(v) }),
            },
            v => v,
        };
        None
    }
    pub fn next_if_eq<'a, T>(self: Pin<&'a mut Self>, t: &T) -> Option<Lend<'a, L>>
    where
        T: for<'all> PartialEq<Lend<'all, L>>,
    {
        self.next_if(|v| t == v)
    }
    /// Drop any peeked value and unpin the lender.
    #[inline]
    pub fn unpin(self: Pin<&mut Self>) -> &mut Self {
        // SAFETY: we're dropping the peeked value and unpinning the lender
        let this = unsafe { self.get_unchecked_mut() };
        this.peeked = None;
        this
    }
}
impl<'this, L> Clone for Peekable<'this, L>
where
    L: Lender + Clone,
{
    fn clone(&self) -> Self { Peekable { lender: self.lender.clone(), peeked: None, _pin: PhantomPinned } }
}
impl<'this, L: fmt::Debug> fmt::Debug for Peekable<'this, L>
where
    L: Lender + fmt::Debug,
    Lend<'this, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Peekable").field("lender", &self.lender).field("peeked", &self.peeked).finish()
    }
}
impl<'lend, 'this, L> Lending<'lend> for Peekable<'this, L>
where
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}
impl<'this, L> Lender for Peekable<'this, L>
where
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> { self.lender.next() }
    #[inline]
    fn count(self) -> usize { self.lender.count() }
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> { self.lender.nth(n) }
    #[inline]
    fn last<'a>(&'a mut self) -> Option<Lend<'a, Self>> { self.lender.last() }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        self.lender.try_fold(init, f)
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.fold(init, f)
    }
}
impl<'this, L: DoubleEndedLender> DoubleEndedLender for Peekable<'this, L> {
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> { self.lender.next_back() }
    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        self.lender.try_rfold(init, f)
    }
    #[inline]
    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.rfold(init, f)
    }
}
impl<'this, L: ExactSizeLender> ExactSizeLender for Peekable<'this, L> {}

impl<'this, L: FusedLender> FusedLender for Peekable<'this, L> {}

impl<'lend, 'this, L> Lending<'lend> for Pin<&mut Peekable<'this, L>>
where
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}
impl<'this, L> Lender for Pin<&mut Peekable<'this, L>>
where
    L: Lender,
{
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: we aren't moving self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        match this.peeked.take() {
            // SAFETY: 'this: 'call
            Some(peeked) => unsafe { core::mem::transmute::<Option<Lend<'this, L>>, Option<Lend<'_, L>>>(peeked) },
            None => this.lender.next(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        // SAFETY: we aren't moving self
        let this = unsafe { self.get_unchecked_mut() };
        let lender = &mut this.lender;
        match this.peeked.take() {
            Some(None) => 0,
            Some(Some(_)) => 1 + lender.count(),
            None => lender.count(),
        }
    }
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        // SAFETY: we aren't moving self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        match this.peeked.take() {
            Some(None) => None,
            // SAFETY: The lend is manually guaranteed to be the only one alive
            Some(v @ Some(_)) if n == 0 => unsafe { core::mem::transmute::<Option<Lend<'this, L>>, Option<Lend<'_, L>>>(v) },
            Some(Some(_)) => this.lender.nth(n - 1),
            None => this.lender.nth(n),
        }
    }
    #[inline]
    fn last<'a>(&'a mut self) -> Option<Lend<'a, Self>> {
        // SAFETY: we aren't moving self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        let peek_opt = match this.peeked.take() {
            Some(None) => return None,
            // SAFETY: 'this: 'call
            Some(v) => unsafe { core::mem::transmute::<Option<Lend<'this, L>>, Option<Lend<'a, L>>>(v) },
            None => None,
        };
        // SAFETY: although we are using &lender when the lend may be &mut lender, we assume that the lend is not modifying the lender in *our* scope
        match this.lender.size_hint().1 {
            Some(n) => this.lender.nth(n.saturating_sub(1)),
            None => peek_opt,
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let this = self.as_ref().get_ref();
        let peek_len = match this.peeked {
            Some(None) => return (0, Some(0)),
            Some(Some(_)) => 1,
            None => 0,
        };
        let (l, r) = this.lender.size_hint();
        (l.saturating_add(peek_len), r.and_then(|r| r.checked_add(peek_len)))
    }
    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        // SAFETY: we aren't moving self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        let acc = match this.peeked.take() {
            Some(None) => return Try::from_output(init),
            Some(Some(v)) => try_!(f(init, v)),
            None => init,
        };
        this.lender.try_fold(acc, f)
    }
    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        // SAFETY: we aren't moving self
        let this = unsafe { self.get_unchecked_mut() };
        let acc = match this.peeked.take() {
            Some(None) => return init,
            Some(Some(v)) => f(init, v),
            None => init,
        };
        this.lender.by_ref().fold(acc, f)
    }
}

impl<'this, L: FusedLender> FusedLender for Pin<&mut Peekable<'this, L>> {}

impl<'this, L> HasNext for Pin<&mut Peekable<'this, L>>
where
    L: Lender,
{
    #[inline]
    fn has_next(&mut self) -> bool { self.as_mut().peek().is_some() }
}
