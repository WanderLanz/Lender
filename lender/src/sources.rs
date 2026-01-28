mod empty;
mod from_fn;
mod from_iter;
mod once;
mod once_with;
mod repeat;
mod repeat_with;
mod windows_mut;

pub use self::{
    empty::{empty, fallible_empty, Empty, FallibleEmpty},
    from_fn::{from_fallible_fn, from_fn, FromFallibleFn, FromFn},
    from_iter::{
        from_fallible_iter, from_into_fallible_iter, from_into_iter, from_iter, lend_fallible_iter,
        lend_iter, FromFallibleIter, FromIntoFallibleIter, FromIntoIter, FromIter,
        LendFallibleIter, LendIter,
    },
    once::{fallible_once, once, FallibleOnce, Once},
    once_with::{fallible_once_with, once_with, FallibleOnceWith, OnceWith},
    repeat::{fallible_repeat, repeat, FallibleRepeat, Repeat},
    repeat_with::{fallible_repeat_with, repeat_with, FallibleRepeatWith, RepeatWith},
    windows_mut::{array_windows_mut, windows_mut, ArrayWindowsMut, WindowsMut, WindowsMutExt},
};
