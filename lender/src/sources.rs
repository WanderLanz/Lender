mod empty;
mod from_fn;
mod from_iter;
mod once;
mod once_with;
mod repeat;
mod repeat_with;
mod windows_mut;

pub use self::{
    empty::{Empty, FallibleEmpty, empty, fallible_empty},
    from_fn::{FromFallibleFn, FromFn, from_fallible_fn, from_fn},
    from_iter::{
        FromFallibleIter, FromIntoFallibleIter, FromIntoIter, FromIter, LendFallibleIter, LendIter,
        from_fallible_iter, from_into_fallible_iter, from_into_iter, from_iter, lend_fallible_iter,
        lend_iter,
    },
    once::{FallibleOnce, Once, fallible_once, once},
    once_with::{FallibleOnceWith, OnceWith, fallible_once_with, once_with},
    repeat::{FallibleRepeat, Repeat, fallible_repeat, repeat},
    repeat_with::{FallibleRepeatWith, RepeatWith, fallible_repeat_with, repeat_with},
    windows_mut::{ArrayWindowsMut, WindowsMut, WindowsMutExt, array_windows_mut, windows_mut},
};
