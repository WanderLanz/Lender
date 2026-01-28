use crate::*;

/// The [`Lender`] version of [`core::iter::ExactSizeIterator`].
pub trait ExactSizeLender: Lender {
    #[inline]
    fn len(&self) -> usize {
        let (lower, upper) = self.size_hint();
        assert_eq!(upper, Some(lower));
        lower
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<I: ExactSizeLender> ExactSizeLender for &mut I {
    #[inline(always)]
    fn len(&self) -> usize {
        (**self).len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }
}

/// The [`FallibleLender`] version of [`core::iter::ExactSizeIterator`].
pub trait ExactSizeFallibleLender: FallibleLender {
    #[inline]
    fn len(&self) -> usize {
        let (lower, upper) = self.size_hint();
        assert_eq!(upper, Some(lower));
        lower
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<I: ExactSizeFallibleLender> ExactSizeFallibleLender for &mut I {
    #[inline(always)]
    fn len(&self) -> usize {
        (**self).len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }
}
