// Test that repeat() and fallible_repeat() reject Lending/FallibleLending types
// that don't implement CovariantLend/CovariantFallibleLend.
// An invariant type could cause UB through the lifetime transmute in next().

use std::cell::Cell;

struct Invariant;

impl<'lend> lender::Lending<'lend> for Invariant {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

struct FallibleInvariant;

impl<'lend> lender::FallibleLending<'lend> for FallibleInvariant {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

fn test_repeat<'a>(v: &'a Cell<Option<&'a String>>)
where
    &'a Cell<Option<&'a String>>: Clone,
{
    let _ = lender::repeat::<Invariant>(v);
}

fn test_fallible_repeat<'a>(v: &'a Cell<Option<&'a String>>)
where
    &'a Cell<Option<&'a String>>: Clone,
{
    let _ = lender::fallible_repeat::<FallibleInvariant, String>(v);
}

fn test_fallible_repeat_err() {
    let _ = lender::fallible_repeat_err::<FallibleInvariant, _>("error".to_string());
}

fn main() {}
