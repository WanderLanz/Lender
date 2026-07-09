// Test that Peekable::peek_mut cannot be called from safe code: it is `unsafe`
// because the returned reference exposes the cached lend with the lender's full
// storage lifetime, allowing a lend to escape its borrow (use-after-free).

use lender::prelude::*;

fn main() {
    let mut p = [1, 2, 3].iter().into_lender().peekable();
    let _ = p.peek_mut(); // ERROR: call to unsafe function is unsafe and requires unsafe block
}
