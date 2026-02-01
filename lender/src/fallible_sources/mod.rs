mod empty;
mod from_fn;
mod from_iter;
mod once;
mod once_with;
mod repeat;
mod repeat_with;

pub use self::{
    empty::{FallibleEmpty, fallible_empty},
    from_fn::{FromFallibleFn, from_fallible_fn},
    from_iter::{
        FromFallibleIter, FromIntoFallibleIter, LendFallibleIter,
        from_fallible_iter, from_into_fallible_iter, lend_fallible_iter,
    },
    once::{FallibleOnce, fallible_once},
    once_with::{FallibleOnceWith, fallible_once_with},
    repeat::{FallibleRepeat, fallible_repeat},
    repeat_with::{FallibleRepeatWith, fallible_repeat_with},
};
