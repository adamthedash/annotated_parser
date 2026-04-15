use num_traits::AsPrimitive;

use crate::combinators::{
    Configured, Configuring, Parameterize,
    delayed::{DelayedValGet, DelayedValSet},
    map, map_silent, try_map,
};
use std::fmt::{Debug, Display};

use crate::{
    Parser,
    combinators::{
        Checkpoint, Cond, Delayed, Map, MapSilent, Opt, RepeatArray, RepeatVec, Trace, TryMap,
        Verify,
    },
};

/// Tail-call adapters for combinators
pub trait ParserAdapter<'a>: Parser<'a> + Sized {
    fn map<F, O>(self, func: F) -> Map<Self, F>
    where
        F: FnMut(Self::Output) -> O,
        O: Debug + Clone + 'static,
    {
        map(self, func)
    }

    fn map_silent<F, O>(self, func: F) -> MapSilent<Self, F>
    where
        F: FnMut(Self::Output) -> O,
        O: Debug + Clone + 'static,
    {
        map_silent(self, func)
    }

    fn try_map<F, O, E>(self, func: F) -> TryMap<Self, F>
    where
        F: FnMut(Self::Output) -> std::result::Result<O, E>,
        O: Debug + Clone + 'static,
        E: Display,
    {
        try_map(self, func)
    }

    fn checkpoint(self) -> Checkpoint<Self> {
        Checkpoint(self)
    }

    fn into_box(self) -> Box<dyn Parser<'a, Input = Self::Input, Output = Self::Output>>
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

    fn delay(self) -> Delayed<Self, Self::Output> {
        Delayed::new(self)
    }

    fn run_if<C>(self, cond: C) -> Cond<C, Self>
    where
        C: DelayedValGet<Value = bool>,
    {
        Cond::new(cond, self)
    }

    fn configured<C>(self, cond: C) -> (Configured<Self>, impl Fn())
    where
        C: DelayedValGet<Value = bool>,
    {
        let parser = Configured::new(self);
        let conf = parser.configure_with(cond);

        (parser, conf)
    }

    fn configuring<F>(self, configurator: F) -> Configuring<Self, F>
    where
        F: Fn(),
    {
        Configuring::new(self, configurator)
    }

    fn optional(self) -> Opt<Self> {
        Opt::new(self)
    }

    fn verify<F>(self, func: F) -> Verify<Self, F>
    where
        F: FnMut(&Self::Output) -> bool,
    {
        Verify::new(self, func)
    }

    fn trace(self, name: impl Into<String>) -> Trace<Self> {
        Trace::new(self, name)
    }

    fn parameterize<V, S>(self, parameters: V, param_input: S) -> Parameterize<S, V, Self>
    where
        S: DelayedValSet,
        S::Value: Clone,
        V: DelayedValGet<Value = Vec<S::Value>>,
    {
        Parameterize::new(parameters, param_input, self)
    }
}

impl<'a, P> ParserAdapter<'a> for P where P: Parser<'a> {}
