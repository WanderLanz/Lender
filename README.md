# `lender` 🙂

[![downloads](https://img.shields.io/crates/d/lender)](https://crates.io/crates/lender)
[![dependents](https://img.shields.io/librariesio/dependents/cargo/lender)](https://crates.io/crates/lender/reverse_dependencies)
[![miri](https://img.shields.io/github/actions/workflow/status/WanderLanz/Lender/test.yml?label=miri)](https://github.com/WanderLanz/Lender/actions/workflows/test.yml)
![license](https://img.shields.io/crates/l/lender)

**Lending Iterator**; a niche, yet seemingly pervasive, antipattern[^1].
This crate provides one such implementation, 'utilizing' [#84533](https://github.com/rust-lang/rust/issues/84533) and [#25860](https://github.com/rust-lang/rust/issues/25860).

Ok, maybe 'antipattern' is a little tough, but let's spare the antagonizing examples, if you can avoid using lending iterators, you *probably* should.
You should heed the counsel of Polonious: "Neither a borrower nor a lender be".

Forewarning, before you go on with this crate, you should consider using a more seasoned 'lending iterator' crate, like the [`lending-iterator`] or [`streaming-iterator`] crates.
Also, if a `dyn Lender` trait object is in your future, this crate **definitely** isn't going to work.
This crate was not made to be used in any sort of production code, so please, use at your own risk (Documentation be damned!).

Nevertheless, to begin, I present to you `WindowsMut`.

```rust
use ::lender::prelude::*;
struct WindowsMut<'a, T> {
    inner: &'a mut [T],
    begin: usize,
    len: usize,
}
impl<'lend, 'a, T> Lending<'lend> for WindowsMut<'a, T> {
    type Lend = &'lend mut [T];
}
impl<'a, T> Lender for WindowsMut<'a, T> {
    fn next<'lend>(&'lend mut self) -> Option<&'lend mut [T]> {
        let begin = self.begin;
        self.begin = self.begin.saturating_add(1);
        self.inner.get_mut(begin..begin + self.len)
    }
}
// Fibonacci sequence
let mut data = vec![0; 3 * 100];
data[1] = 1;
WindowsMut { slice: &mut data, begin: 0, len: 3 }.for_each(|w: &mut [f32]| w[2] = w[0] + w[1]);
```

As all great standard examples are, a `WindowsMut` just for a Fibonacci sequence is actually a great example of what you **shouldn't** use lending iterators for.
Libraries can just provide `Index` and `IndexMut` on their collections and it's a lot of boilerplate for something a simple for loop can do.

```rust
let mut data = vec![0; 3 * 100];
data[1] = 1;
for i in 2..data.len() {
    data[i] = data[i - 1] + data[i - 2];
}
```

So, let's look at a slightly more interesting example, `LinesStr`, an `io::Lines` with an `Item` of `&str` instead of `String`.
It's a good example of borrowing from the iterator itself.

```rust
use std::io;
use ::lender::prelude::*;
struct LinesStr<B> {
    buf: B,
    line: String,
}
impl<B: io::BufRead, 'lend> Lending<'lend> for LinesStr<B> {
    type Lend = io::Result<&'lend str>;
}
impl<B: io::BufRead> Lender for LinesStr<B> {
    fn next<'lend>(&'lend mut self) -> Option<io::Result<&'lend str>> {
        self.line.clear();
        match self.buf.read_line(&mut self.line) {
            Err(e) => return Some(Err(e)),
            Ok(0) => return None,
            Ok(_nread) => (),
        };
        if self.line.ends_with('\n') {
            self.line.pop();
            if self.line.ends_with('\r') {
                self.line.pop();
            }
        }
        Some(Ok(&self.line))
    }
}
```

For most cases like this, you could just probably rely on the optimizer, i.e. reusing the same buffer instead of allocating a new one each time,
but you see where we're going with this.

Turn a lender into an iterator with `cloned()` where lend is `Clone`, `copied()` where lend is `Copy`, `owned()` where lend is `ToOwned`, or `iter()` where lend already satisfies the restrictions of `Iterator::Item`.

`partition_in_place` and `array_chunks` are unsupported.

## Resources

Please thank and check out the great resources below that helped me and many others learn about Rust and the lending iterator problem.

- [Sabrina Jewson's Blog](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats) for their awesome
blog post on why lifetime GATs are not (yet) the solution to this problem, I highly recommend reading it.
- The awesome people on the [Rust Users Forum](https://users.rust-lang.org/) in helping me understand the borrow checker and HRTBs better
and being patient with me and other aspiring rustaceans as we try to learn more about Rust.
- [Daniel Henry-Mantilla](https://github.com/danielhenrymantilla) for writing [`lending-iterator`] and many other great crates and sharing their great work.
- Everyone who's contributed to Rust for making such a great language and iterator library.

<!-- markdownlint-disable MD026 -->
## Unsafe & Transmutes Beware!!!

Many patterns in lending iterators require polonius-emulating unsafe code, but please, if you see any unsafe code that can be made safe, please let me know! I am still learning Rust and I'm sure I've made many mistakes.

## License

Licensed under either the [MIT](/LICENSE-MIT.txt) or [Apache-2.0](/LICENSE-APACHE.txt) license.

[^1]: An [antipattern](https://en.wikipedia.org/wiki/Anti-pattern) is a common response to a recurring problem that is usually ineffective and risks being highly counterproductive.

[`lending-iterator`]: https://crates.io/crates/lending-iterator
[`streaming-iterator`]: https://crates.io/crates/streaming-iterator
