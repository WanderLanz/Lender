// Test that lend!() rejects complex patterns that might hide invariance.
// Users should use covariant_lend!() for these cases.

use std::cell::Cell;

// Generic with nested lifetime - potentially invariant
type A = lender::lend!(&'lend Cell<&'lend String>);

// Path type (not a simple ident)
type B = lender::lend!(&'lend std::string::String);

// Tuple with reference inside (not supported)
type C = lender::lend!((&'lend i32, &'lend i32));

fn main() {}
