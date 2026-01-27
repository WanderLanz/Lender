// Test that covariant_lend_fallible! correctly rejects invariant types.

use std::cell::Cell;

use lender::covariant_lend_fallible;

// This should fail to compile because the type is invariant in 'lend
covariant_lend_fallible!(InvariantLend = &'lend Cell<Option<&'lend String>>);

fn main() {}
