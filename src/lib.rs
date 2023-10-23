//! # Lender
//!
//! **Lending Iterator**: a niche, yet seemingly pervasive, antipattern.
//! This crate provides one such implementation, 'utilizing' [#84533](https://github.com/rust-lang/rust/issues/84533) and [#25860](https://github.com/rust-lang/rust/issues/25860).
//!
//! Ok, maybe 'antipattern' is a little tough, but let's spare the antagonizing examples, if you can avoid using lending iterators, you generally should.
//! You should heed the counsel of Polonious: "Neither a borrower nor a lender be".
//!
//! Forewarning, before you go on with this crate, you should consider using a more seasoned 'lending iterator' crate, like the [`lending-iterator`] or [`streaming-iterator`] crates.
//! Also, if a `dyn Lender` trait object is in your future, this crate **definitely** isn't going to work.
//! This crate was not made to be used in any sort of production code, so please, use at your own risk (Documentation be damned! Unsafe transmutes beware!).
//!
//! Commonly known as a "lending iterator", a lender is a kind of iterator over items
//! that live at least as long as the mutable reference to the lender in a call to [`Lender::next()`].
//! In other words, a kind of iterator that lends an item one at a time, a pattern not implementable
//! by the current definition of [`Iterator`] which only encompasses iterators over items that live at least
//! as long as the iterators themselves, i.e. `Self: 'this` implies `<Self as Iterator>::Item: 'this`.
//!
//! ## Examples
//!
//! I present to you `WindowsMut`.
//!
//! ```rust
//! use ::lender::prelude::*;
//!
//! struct WindowsMut<'a, T> {
//!     slice: &'a mut [T],
//!     begin: usize,
//!     len: usize,
//! }
//! impl<'lend, 'a, T> Lending<'lend> for WindowsMut<'a, T> {
//!     type Lend = &'lend mut [T];
//! }
//! impl<'a, T> Lender for WindowsMut<'a, T> {
//!     fn next<'lend>(&'lend mut self) -> Option<&'lend mut [T]> {
//!         let begin = self.begin;
//!         self.begin = self.begin.saturating_add(1);
//!         self.slice.get_mut(begin..begin + self.len)
//!     }
//! }
//! // Fibonacci sequence
//! let mut data = vec![0u32; 9];
//! data[1] = 1;
//! WindowsMut { slice: &mut data, begin: 0, len: 3 }
//!     .for_each(hrc_mut!(for<'lend> |w: &'lend mut [u32]| { w[2] = w[0] + w[1] }));
//! assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
//! ```
//!
//! As all great standard examples are, a `WindowsMut` just for a Fibonacci sequence is actually a great example of what you **shouldn't** use lending iterators for.
//! Libraries can just provide [`Index`](core::ops::Index) and [`IndexMut`](core::ops::IndexMut) on their collections and it's a lot of boilerplate for something a simple for loop can do.
//!
//! ```rust
//! let mut data = vec![0; 9];
//! data[1] = 1;
//! for i in 2..data.len() {
//!     data[i] = data[i - 1] + data[i - 2];
//! }
//! assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
//! ```
//!
//! So, let's look at a slightly more interesting example, `LinesStr`, an [`std::io::Lines`] with an `Item` of `&str` instead of `String`.
//! It's a good example of borrowing from the iterator itself.
//!
//! ```rust
//! use std::io;
//! use ::lender::prelude::*;
//!
//! struct LinesStr<B> {
//!     buf: B,
//!     line: String,
//! }
//! impl<'lend, B: io::BufRead> Lending<'lend> for LinesStr<B> {
//!     type Lend = io::Result<&'lend str>;
//! }
//! impl<B: io::BufRead> Lender for LinesStr<B> {
//!     fn next<'lend>(&'lend mut self) -> Option<io::Result<&'lend str>> {
//!         self.line.clear();
//!         match self.buf.read_line(&mut self.line) {
//!             Err(e) => return Some(Err(e)),
//!             Ok(0) => return None,
//!             Ok(_nread) => (),
//!         };
//!         if self.line.ends_with('\n') {
//!             self.line.pop();
//!             if self.line.ends_with('\r') {
//!                 self.line.pop();
//!             }
//!         }
//!         Some(Ok(&self.line))
//!     }
//! }
//!
//! let buf = io::BufReader::with_capacity(10, "Hello\nWorld\n".as_bytes());
//! let mut lines = LinesStr { buf, line: String::new() };
//! assert_eq!(lines.next().unwrap().unwrap(), "Hello");
//! assert_eq!(lines.next().unwrap().unwrap(), "World");
//! ```
//!
//! For *most* cases like this, you could just probably rely on the optimizer, i.e. reusing the same buffer instead of allocating a new one each time,
//! but you see where we're going with this.
//!
//! ## Implementing Lender
//!
//! To implement [`Lender`], first you'll need to implement the [`Lending`] trait for your type.
//! This is the equivalent provider of [`Iterator::Item`] for [`Lender`]s.
//!
//! ```rust
//! use ::lender::prelude::*;
//! struct StrRef<'a>(&'a str);
//! impl<'this, 'lend> Lending<'lend> for StrRef<'this> {
//!     type Lend = &'lend str;
//! }
//! ```
//!
//! The lifetime parameter `'lend` describes the lifetime of the `Lend`.
//! It works by using a default generic of `&'lend Self` which induces an implicit reference lifetime bound `'lend: 'this`,
//! necessary for usage of higher-ranked trait bounds with `Lend`.
//!
//! Next, you'll need to implement the [`Lender`] trait for your type, the lending equivalent of [`Iterator`].
//!
//! ```rust
//! use ::lender::prelude::*;
//! struct StrRef<'a>(&'a str);
//! impl<'this, 'lend> Lending<'lend> for StrRef<'this> {
//!     type Lend = &'lend str;
//! }
//! impl<'this> Lender for StrRef<'this> {
//!     fn next<'lend>(&'lend mut self) -> Option<&'lend str> {
//!         Some(self.0)
//!     }
//! }
//! ```
//!
//! [`Lender`] provides all of the methods as [`Iterator`], except [`Iterator::partition_in_place`] and [`Iterator::array_chunks`],
//! and most provide the same functionality as the equivalent [`Iterator`] method.
//!
//! Notable differences in behavior include [`Lender::next_chunk`] providing a lender instead of an array
//! and certain closures may require usage of the [`hrc!`], [`hrc_mut!`], [`hrc_once!`] (higher-ranked closure) macros,
//! which provide a stable replacement for the `closure_lifetime_binder` feature.
//!
//! To provide a similar functionality to [`Iterator::array_chunks`], the [`Lender::chunky`] method makes lenders nice and chunky 🙂.
//!
//! Turn a lender into an iterator with [`Lender::cloned()`] where lend is [`Clone`], [`Lender::copied()`] where lend is [`Copy`],
//! [`Lender::owned()`] where lend is [`ToOwned`](alloc::borrow::ToOwned),or [`Lender::iter()`] where the lender already satisfies the restrictions of [`Iterator`].
//!
//! ## Resources
//!
//! Please check out the great resources below that helped me and many others learn about Rust and the lending iterator problem. Thank you to everyone!
//!
//! - [Sabrina Jewson's Blog](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats) for their awesome
//! blog post on why lifetime GATs are not (yet) the solution to this problem, I highly recommend reading it.
//! - The awesome people on the [Rust Users Forum](https://users.rust-lang.org/) in helping me understand the borrow checker and HRTBs better
//! and being patient with me and other aspiring rustaceans as we try to learn more about Rust.
//! - [Daniel Henry-Mantilla](https://github.com/danielhenrymantilla) for writing [`lending-iterator`] and many other great crates and sharing their great work.
//! - Everyone who's contributed to Rust for making such a great language and iterator library.
//!
//! [`lending-iterator`]: https://crates.io/crates/lending-iterator
//! [`streaming-iterator`]: https://crates.io/crates/streaming-iterator

#![no_std]

extern crate alloc;

#[cfg(doctest)]
#[allow(non_camel_case_types)]
/// ```rust,compile_fail
/// struct WindowsMut<'a, T> {
///     inner: &'a mut [T],
///     begin: usize,
///     len: usize,
/// }
/// impl<'a, T> Iterator for WindowsMut<'a, T> {
///     type Item = &mut [T];
///
///     fn next(&mut self) -> Option<Self::Item> {
///         let begin = self.begin;
///         self.begin = self.begin.saturating_add(1);
///         self.inner.get_mut(begin..begin + self.len)
///     }
/// }
/// ```
pub struct _Lender_Doctest_Sanity_Check;

mod private {
    pub trait Sealed {}
    pub struct Seal<T>(T);
    impl<T> Sealed for Seal<T> {}
}
pub(crate) use private::{Seal, Sealed};

mod adapters;
pub use adapters::*;
mod traits;
pub use traits::*;
pub mod higher_order;
mod sources;
pub use sources::*;
pub mod try_trait_v2;

pub mod prelude {
    pub use crate::{
        hrc, hrc_mut, hrc_once, lend, DoubleEndedLender, ExactSizeLender, ExtendLender, FromLender, IntoLender, Lend,
        Lender, Lending,
    };
}
