// Test that covariant_fallible_lend! correctly rejects invariant types.

use std::cell::Cell;

use lender::covariant_fallible_lend;

// This should fail to compile because the type is invariant in 'lend
covariant_fallible_lend!(InvariantLend = &'lend Cell<Option<&'lend String>>);

fn main() {}
