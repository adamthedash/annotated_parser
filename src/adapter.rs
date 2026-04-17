use num_traits::AsPrimitive;

use crate::combinators::{
    Configured, Configuring, Many, Parameterize, ParserTuple, Peek, Preceded, SeparatedArray,
    SeparatedTuple, Surrounded, SurroundedSymmetrical, Terminated, TraceOpaque,
    delayed::{DelayedValGet, DelayedValSet},
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
pub trait ParserAdapter<Input>: Parser<Input> + Sized {
    fn map<F, O>(self, func: F) -> Map<Self, F>
    where
        F: FnMut(Self::Output) -> O,
        O: Debug + Clone + 'static,
    {
        Map::new(self, func)
    }

    fn map_silent<F, O>(self, func: F) -> MapSilent<Self, F>
    where
        F: FnMut(Self::Output) -> O,
        O: Debug + Clone + 'static,
    {
        MapSilent::new(self, func)
    }

    fn try_map<F, O, E>(self, func: F) -> TryMap<Self, F>
    where
        F: FnMut(Self::Output) -> std::result::Result<O, E>,
        O: Debug + Clone + 'static,
        E: Display,
    {
        TryMap::new(self, func)
    }

    fn checkpoint(self) -> Checkpoint<Self>
    where
        Input: Copy,
    {
        Checkpoint::new(self)
    }

    fn peek(self) -> Peek<Self>
    where
        Input: Copy,
    {
        Peek::new(self)
    }

    fn repeat<const N: usize>(self) -> RepeatArray<Self, [Self::Output; N]> {
        RepeatArray::new(self)
    }

    fn repeat_vec<C>(self, count: C) -> RepeatVec<Self, C>
    where
        C: DelayedValGet,
        C::Value: AsPrimitive<usize>,
    {
        RepeatVec::new(self, count)
    }

    fn many(self) -> Many<Self>
    where
        Input: Copy,
    {
        Many::new(self)
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

    fn optional(self) -> Opt<Self>
    where
        Input: Copy,
    {
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

    fn trace_opaque(self, name: impl Into<String>) -> TraceOpaque<Self> {
        TraceOpaque::new(self, name)
    }

    fn parameterize<V, S>(self, parameters: V, param_input: S) -> Parameterize<S, V, Self>
    where
        S: DelayedValSet,
        S::Value: Clone,
        V: DelayedValGet<Value = Vec<S::Value>>,
    {
        Parameterize::new(parameters, param_input, self)
    }

    fn surrounded_by<L, R>(self, left: L, right: R) -> Surrounded<L, Self, R>
    where
        L: Parser<Input>,
        R: Parser<Input>,
    {
        Surrounded::new(left, self, right)
    }

    fn surrounded_by_sym<O>(self, outer: O) -> SurroundedSymmetrical<Self, O>
    where
        O: Parser<Input>,
    {
        SurroundedSymmetrical::new(self, outer)
    }

    fn non_empty<T>(self) -> Verify<Self, impl FnMut(&Self::Output) -> bool>
    where
        Self: Parser<Input, Output = Vec<T>>,
    {
        self.verify(|values| !values.is_empty())
    }

    fn separated<const N: usize, S>(
        self,
        separator: S,
    ) -> SeparatedArray<Self, S, [Self::Output; N]>
    where
        S: Parser<Input>,
    {
        SeparatedArray::new(separator, self)
    }

    fn separated_tuple<S>(self, separator: S) -> SeparatedTuple<S, Self>
    where
        Self: ParserTuple<Input>,
        S: Parser<Input>,
    {
        SeparatedTuple::new(separator, self)
    }

    fn ignore_then<P>(self, keep: P) -> Preceded<Self, P>
    where
        P: Parser<Input>,
    {
        Preceded::new(self, keep)
    }

    fn then_ignore<P>(self, ignore: P) -> Terminated<P, Self>
    where
        P: Parser<Input>,
    {
        Terminated::new(self, ignore)
    }
}

impl<Input, P> ParserAdapter<Input> for P where P: Parser<Input> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_many1() {
        fn create_parser() -> impl for<'a> Parser<&'a [u8], Output = Vec<u8>> {
            u8::LE.many().non_empty()
        }

        fn use_parser() -> (Vec<u8>, Vec<u8>) {
            let mut parser = create_parser();

            let input = vec![0; 5];
            let (value, _) = parser.parse(&mut input.as_slice()).unwrap();

            (input, value)
        }

        use_parser();
    }
}
