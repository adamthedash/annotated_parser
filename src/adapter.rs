use num_traits::AsPrimitive;

use crate::combinators::delayed::DelayedValGet;
use std::fmt::{Debug, Display};

use crate::{
    Parser,
    combinators::{
        Checkpoint, Cond, Delayed, Map, MapSilent, Opt, RepeatArray, RepeatVec, Trace, TryMap,
        Verify,
    },
};

/// Tail-call adapters for combinators
pub trait ParserAdapter: Parser + Sized {
    fn map<F, O>(self, func: F) -> Map<Self, F>
    where
        F: FnMut(Self::Output) -> O,
        O: Debug,
    {
        Map::new(self, func)
    }

    fn map_silent<F, O>(self, func: F) -> MapSilent<Self, F>
    where
        F: FnMut(Self::Output) -> O,
        O: Debug,
    {
        MapSilent::new(self, func)
    }

    fn try_map<F, O, E>(self, func: F) -> TryMap<Self, F>
    where
        F: FnMut(Self::Output) -> std::result::Result<O, E>,
        O: Debug,
        E: Display,
    {
        TryMap::new(self, func)
    }

    fn checkpoint(self) -> Checkpoint<Self> {
        Checkpoint(self)
    }

    fn into_box(self) -> Box<dyn Parser<Output = Self::Output>>
    where
        Self: 'static,
    {
        Box::new(self)
    }

    fn repeat<const N: usize>(self) -> RepeatArray<Self, [Self::Output; N]> {
        RepeatArray::new(self)
    }

    fn repeat_vec<C, V>(self, count: C) -> RepeatVec<Self, C>
    where
        C: DelayedValGet<Value = V>,
        V: AsPrimitive<usize>,
    {
        RepeatVec::new(self, count)
    }

    fn delay(self) -> Delayed<Self> {
        Delayed::new(self)
    }

    fn run_if<D>(self, value: D, cond: fn(&D::Value) -> bool) -> Cond<D, Self>
    where
        D: DelayedValGet,
    {
        Cond::new(value, cond, self)
    }

    fn optional(self) -> Opt<Self> {
        Opt(self)
    }

    fn verify<F>(self, func: F) -> Verify<Self, F>
    where
        F: FnMut(&Self::Output) -> bool,
        Self::Output: Debug,
    {
        Verify::new(self, func)
    }

    fn trace(self, name: impl Into<String>) -> Trace<Self> {
        Trace::new(self, name)
    }
}

impl<P> ParserAdapter for P where P: Parser {}
