mod empty;
mod from_fn;
mod from_iter;
mod once;
mod once_with;
mod repeat;
mod repeat_with;

pub use self::{
    empty::{Empty as FallibleEmpty, empty as fallible_empty},
    from_fn::{FromFn as FromFallibleFn, from_fn as from_fallible_fn},
    from_iter::{
        FromIntoIter as FromIntoFallibleIter, FromIter as FromFallibleIter,
        LendIter as LendFallibleIter, from_into_iter as from_into_fallible_iter,
        from_iter as from_fallible_iter, lend_iter as lend_fallible_iter,
    },
    once::{Once as FallibleOnce, once as fallible_once},
    once_with::{OnceWith as FallibleOnceWith, once_with as fallible_once_with},
    repeat::{Repeat as FallibleRepeat, repeat as fallible_repeat},
    repeat_with::{RepeatWith as FallibleRepeatWith, repeat_with as fallible_repeat_with},
};
