// Test that lend!() rejects complex patterns that might hide invariance.
// Users should use covariant_lend!() for these cases.

use std::cell::Cell;

// Generic with nested lifetime - potentially invariant
type A = lender::lend!(&'lend Cell<&'lend String>);

// Generic type (not a simple ident)
type B = lender::lend!(&'lend Vec<u8>);

// Path type (not a simple ident)
type C = lender::lend!(&'lend std::string::String);

// Tuple with reference inside (not supported)
type D = lender::lend!((&'lend u8, &'lend i32));

fn main() {}
