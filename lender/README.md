# `lender` 🙂

[![downloads](https://img.shields.io/crates/d/lender)](https://crates.io/crates/lender)
[![dependents](https://img.shields.io/librariesio/dependents/cargo/lender)](https://crates.io/crates/lender/reverse_dependencies)
[![miri](https://img.shields.io/github/actions/workflow/status/WanderLanz/Lender/test.yml?label=miri)](https://github.com/WanderLanz/Lender/actions/workflows/test.yml)
![license](https://img.shields.io/crates/l/lender)

A _lender_, also called a _lending iterator_, is an iterator that lends mutable borrows 
to the items it returns. In particular, this means that the reference to an item is 
invalidated by the subsequent call to `next`. Niko Matsakis has an interesting 
[blog post](https://smallcultfollowing.com/babysteps/blog/2023/05/09/giving-lending-and-async-closures/) 
explaining a general view of giving vs. lending traits.

The typical example that cannot be written with standard Rust iterators, 
but is covered by lenders, is that of a lender returning [mutable, overlapping windows of a slice]().

But lenders are more general than that, as they might return items that depend on
some mutable state stored in the iterator. For example, a lender might 
return references to the lines of a file reusing an internal buffer; also, 
starting from an iterator on pairs of integers lexicographically sorted, a lender 
might return iterators on pairs with the same first coordinate without any copying; 
clearly, in all these cases any call on `next` would invalidate the reference
returned by the previous call.

This crate provides a lender trait and an associated library of utility methods,
“utilizing” [#84533](https://github.com/rust-lang/rust/issues/84533) and [#25860](https://github.com/rust-lang/rust/issues/25860)
to implement the [lender design based on higher-rank trait bounds proposed by Sabrina Jewson](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats).

Similarly to what happens with standard iterators, besides the fundamental 
[`Lender`](https://docs.rs/lender/latest/lender/trait.Lender.html) trait there is an
[`IntoLender`](https://docs.rs/lender/latest/lender/trait.IntoLender.html)
trait, and methods such as [`for_each`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.for_each). 

Indeed, the crate implements for [`Lender`](https://docs.rs/lender/latest/lender/trait.Lender.html)
all of the methods as `Iterator`, except `Iterator::partition_in_place` and `Iterator::array_chunks`,
and most provide the same functionality as the equivalent `Iterator` method.

Notable differences in behavior include `Lender::next_chunk` providing a lender instead of an array
and certain closures requiring usage of the `hrc!`, `hrc_mut!`, `hrc_once!` (higher-ranked closure) macros,
which provide a stable replacement for the `closure_lifetime_binder` feature.

To provide similar functionality to `Iterator::array_chunks`, the 
[`chunky`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.chunky) method makes lenders nice and chunky 🙂.

Turn a lender into an iterator with [`cloned`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.cloned) 
where lend is `Clone`, [`copied`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.copied) where lend is `Copy`,
[`owned`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.owned) where lend is `ToOwned`, or 
[`iter`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.iter) where the lender already satisfies the restrictions of `Iterator`.

## Usage

The Rust `for` syntax for iterating over types implementing `IntoIterator` will not work with lenders. The idiomatic way
of iterating over a lender is to use a `while let` loop, as in:

```text
while let Some(item) = lender.next() {
    // Do something with item
}
```
Note that the expression after the equal sign cannot be a method call returning a lender,
as you would iterate over the first element forever.

To simplify usage, we provide a function-like procedural macro 
[`for_!`](https://docs.rs/lender-derive/latest/lender_derive/macro.for_.html) that makes it 
possible to use a `for`-like syntax with types implementing `IntoLender`:
```text
for_!(item in into_lender {
    // Do something with item
});
```

Finally, we can use the `for_each` method, which takes a closure as argument, but managing lifetime in closures can be 
challenging:
```text
lender.for_each{
    hrc_mut!(for<'lend> |item: &'lend mut TYPE| {
        // do something with item of type TYPE 
    })
};
```

## Caveats

Forewarning, before you go on with this crate, you should consider using a more seasoned 
crate, like [`lending-iterator`], which, however, does not use directly higher-rank trait bounds,
but rather relies on simulating them using macros.

Also, if a `dyn Lender` trait object is in your future, this crate **definitely** isn't going to work.
This crate was not made to be used in any sort of production code, so please, use 
it at your own risk (Documentation be damned! Unsafe transmutes beware!).

Finally, note that, as a general rule, if you can avoid using lenders, you should.
You should heed the counsel of Polonius: “Neither a borrower nor a lender be”.

## Examples

Let us compute the Fibonacci numbers using mutable windows:

```rust
use ::lender::prelude::*;
use lender_derive::for_;

// Fibonacci sequence
let mut data = vec![0u32; 3 * 3];
data[1] = 1;

// Fibonacci sequence, most ergonomic usage: for_! procedural macro.
for_!(w in data.array_windows_mut::<3>() {
   w[2] = w[0] + w[1];
});
assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);

// Fibonacci sequence, explicit while let loop: you MUST assign the lender to a variable.
let mut windows = data.array_windows_mut::<3>();
while let Some(w) = windows.next() {
   w[2] = w[0] + w[1];
}
assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);

// Fibonacci sequence, for_each with hrc_mut!
data.array_windows_mut::<3>()
    .for_each(hrc_mut!(for<'lend> |w: &'lend mut [u32; 3]| {
         w[2] = w[0] + w[1]
    }));
assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
```
This is a quite contrived example, but it shows how lenders can be used to mutate a slice in place.

So, let's look at a slightly more interesting example, `LinesStr`, an `io::Lines` with an `Item` of `&str` instead of `String`.
It's a good example of borrowing from the iterator itself.

```rust
use std::io;
use ::lender::prelude::*;

struct LinesStr<B> {
    buf: B,
    line: String,
}
impl<'lend, B: io::BufRead> Lending<'lend> for LinesStr<B> {
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

let buf = io::BufReader::with_capacity(10, "Hello\nWorld\n".as_bytes());
let mut lines = LinesStr { buf, line: String::new() };
assert_eq!(lines.next().unwrap().unwrap(), "Hello");
assert_eq!(lines.next().unwrap().unwrap(), "World");
```

## Implementing Lender

To implement `Lender`, first you'll need to implement the `Lending` trait for your type.
This is the equivalent provider of `Iterator::Item` for `Lender`s.

```rust
use ::lender::prelude::*;
struct StrRef<'a>(&'a str);
impl<'this, 'lend> Lending<'lend> for StrRef<'this> {
    type Lend = &'lend str;
}
```

The lifetime parameter `'lend` describes the lifetime of the `Lend`.
It works by using a default generic of `&'lend Self` which induces an implicit reference lifetime bound `'lend: 'this`,
necessary for usage of higher-ranked trait bounds with `Lend`.

Next, you'll need to implement the `Lender` trait for your type, the lending equivalent of `Iterator`.

```rust
use ::lender::prelude::*;
struct StrRef<'a>(&'a str);
impl<'this, 'lend> Lending<'lend> for StrRef<'this> {
    type Lend = &'lend str;
}
impl<'this> Lender for StrRef<'this> {
    fn next<'lend>(&'lend mut self) -> Option<&'lend str> {
        Some(self.0)
    }
}
```

## Type-inference problems

Due to the complex type dependencies and higher-kind trait bounds
involved, the current Rust compiler cannot
always infer the correct type of a lender and of the items it returns.
In general, when writing methods accepting a [`Lender`]
restricting the returned item type with a *type* will work, as in:

```rust
use lender::*;

struct MockLender {}

impl<'lend> Lending<'lend> for MockLender {
    type Lend = &'lend str;
}

impl Lender for MockLender {
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn read_lender<L>(lender: L)
where
    L: Lender + for<'lend> Lending<'lend, Lend = &'lend str>,
{}

fn test_mock_lender(m: MockLender) {
    read_lender(m);
}
```

However, the following code, which restricts the returned items using a trait bound,
does not compile as of Rust 1.73.0:

```ignore
use lender::*;

struct MockLender {}

impl<'lend> Lending<'lend> for MockLender {
    type Lend = &'lend str;
}

impl Lender for MockLender {
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn read_lender<L>(lender: L)
where
    L: Lender,
    for<'lend> Lend<'lend, L>: AsRef<str>,
{}

fn test_mock_lender(m: MockLender) {
    read_lender(m);
}
```

The workaround is to use an explicit type annotation:

```rust
use lender::*;

struct MockLender {}

impl<'lend> Lending<'lend> for MockLender {
    type Lend = &'lend str;
}

impl Lender for MockLender {
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn read_lender<L>(lender: L)
where
    L: Lender,
    for<'lend> Lend<'lend, L>: AsRef<str>,
{}

fn test_mock_lender(m: MockLender) {
    read_lender::<MockLender>(m);
}
```

## Resources

Please check out the great resources below that helped me and many others learn about Rust and the lending iterator problem. Thank you to everyone!

- [Sabrina Jewson's Blog](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats) for their awesome
blog post on why lifetime GATs are not (yet) the solution to this problem, I highly recommend reading it.
- The awesome people on the [Rust Users Forum](https://users.rust-lang.org/) in helping me understand the borrow checker and HRTBs better
and being patient with me and other aspiring rustaceans as we try to learn more about Rust.
- [Daniel Henry-Mantilla](https://github.com/danielhenrymantilla) for writing [`lending-iterator`] and many other great crates and sharing their great work.
- Everyone who's contributed to Rust for making such a great language and iterator library.
- There is a GAT-based implementation at [`gat-lending-iterator`].

<!-- markdownlint-disable MD026 -->
## Unsafe & Transmutes Beware!!!

Many patterns in lenders require polonius-emulating unsafe code, but please, if you see any unsafe code that can be made safe, please let me know!

[`lending-iterator`]: https://crates.io/crates/lending-iterator
[`streaming-iterator`]: https://crates.io/crates/streaming-iterator
[`gat-lending-iterator`]: https://crates.io/crates/gat-lending-iterator
