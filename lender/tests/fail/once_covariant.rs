// Test that once() and fallible_once() reject Lending/FallibleLending types
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

fn test_once<'a>(v: &'a Cell<Option<&'a String>>) {
    let _ = lender::once::<Invariant>(v);
}

fn test_fallible_once<'a>(v: Result<&'a Cell<Option<&'a String>>, std::io::Error>) {
    let _ = lender::fallible_once::<FallibleInvariant, std::io::Error>(v);
}

fn main() {}
