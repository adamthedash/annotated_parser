use num_traits::AsPrimitive;

use crate::combinators::{
    Configured, Configuring, Dispatch, Many, ParameterInput, Parameterize, Parameters, ParserTuple,
    Peek, Preceded, RepeatTillExc, RepeatTillInc, SameParserTuple, SeparatedArray, SeparatedTuple,
    SeparatedVec, Surrounded, SurroundedSymmetrical, Terminated, TraceOpaque,
};
use crate::{ForwardRefGet, ParserOutput};
use std::fmt::Display;

use crate::{
    Parser,
    combinators::{
        Checkpoint, Cond, Map, MapSilent, Opt, RepeatArray, RepeatVec, Store, Trace, TryMap, Verify,
    },
};

/// Tail-call adapters for combinators
pub trait ParserAdapter<Input>: Parser<Input> + Sized {
    /// Apply an infallible function to the output.
    ///
    /// See [`Map`] for more info.
    fn map<F, O>(self, func: F) -> Map<Self, F>
    where
        F: FnMut(Self::Output) -> O,
        O: ParserOutput,
    {
        Map::new(self, func)
    }

    /// Apply an infallible function to the output without adding to the trace.
    ///
    /// See [`MapSilent`] for more info.
    fn map_silent<F, O>(self, func: F) -> MapSilent<Self, F>
    where
        F: FnMut(Self::Output) -> O,
        O: ParserOutput,
    {
        MapSilent::new(self, func)
    }

    /// Apply a fallible function to the output.
    ///
    /// See [`TryMap`] for more info.
    fn try_map<F, O, E>(self, func: F) -> TryMap<Self, F>
    where
        F: FnMut(Self::Output) -> std::result::Result<O, E>,
        O: ParserOutput,
        E: Display,
    {
        TryMap::new(self, func)
    }

    /// Restore input position on failure.
    ///
    /// See [`Checkpoint`] for more info.
    fn checkpoint(self) -> Checkpoint<Self>
    where
        Input: Copy,
    {
        Checkpoint::new(self)
    }

    /// Look ahead without consuming input.
    ///
    /// See [`Peek`] for more info.
    fn peek(self) -> Peek<Self>
    where
        Input: Copy,
    {
        Peek::new(self)
    }

    /// Repeat the parser a fixed number of times.
    ///
    /// See [`RepeatArray`] for more info.
    fn repeat<const N: usize>(self) -> RepeatArray<Self, [Self::Output; N]> {
        RepeatArray::new(self)
    }

    /// Repeat the parser a runtime-determined number of times.
    ///
    /// See [`RepeatVec`] for more info.
    fn repeat_vec<C>(self, count: C) -> RepeatVec<Self, C>
    where
        C: ForwardRefGet,
        C::Value: AsPrimitive<usize>,
    {
        RepeatVec::new(self, count)
    }

    /// Apply the parser repeatedly until the terminator succeeds, without consuming it.
    ///
    /// See [`RepeatTillExc`] for more info.
    fn repeat_till_exc<T>(self, terminator: T) -> RepeatTillExc<Self, T>
    where
        T: Parser<Input>,
        Input: Copy,
    {
        RepeatTillExc::new(self, terminator)
    }

    /// Apply the parser repeatedly until the terminator succeeds, consuming it.
    ///
    /// See [`RepeatTillInc`] for more info.
    fn repeat_till_inc<T>(self, terminator: T) -> RepeatTillInc<Self, T>
    where
        T: Parser<Input>,
        Input: Copy,
    {
        RepeatTillInc::new(self, terminator)
    }

    /// Apply the parser repeatedly until it fails.
    ///
    /// See [`Many`] for more info.
    fn many(self) -> Many<Self>
    where
        Input: Copy,
    {
        Many::new(self)
    }

    /// Store the parser's output in a `ForwardRef`.
    ///
    /// See [`Store`] for more info.
    fn store(self) -> Store<Self, Self::Output> {
        Store::new(self)
    }

    /// Conditionally run the parser based on a boolean value.
    ///
    /// See [`Cond`] for more info.
    fn run_if<C>(self, cond: C) -> Cond<C, Self>
    where
        C: ForwardRefGet<Value = bool>,
    {
        Cond::new(cond, self)
    }

    /// Create a parser that can be externally enabled or disabled.
    ///
    /// See [`Configured`] for more info.
    fn configured<C>(self, cond: C) -> (Configured<Self>, impl Fn())
    where
        C: ForwardRefGet<Value = bool>,
    {
        let parser = Configured::new(self);
        let conf = parser.configure_with(cond);

        (parser, conf)
    }

    /// Run the parser, then execute a side-effect closure.
    ///
    /// See [`Configuring`] for more info.
    fn configuring<F>(self, configurator: F) -> Configuring<Self, F>
    where
        F: Fn(),
    {
        Configuring::new(self, configurator)
    }

    /// Optionally apply the parser.
    ///
    /// See [`Opt`] for more info.
    fn optional(self) -> Opt<Self>
    where
        Input: Copy,
    {
        Opt::new(self)
    }

    /// Validate the parser's output with a predicate.
    ///
    /// See [`Verify`] for more info.
    fn verify<F>(self, func: F) -> Verify<Self, F>
    where
        F: FnMut(&Self::Output) -> bool,
    {
        Verify::new(self, func)
    }

    /// Add a user-friendly name to the parser's trace.
    ///
    /// See [`Trace`] for more info.
    fn trace(self, name: impl Into<String>) -> Trace<Self> {
        Trace::new(self, name)
    }

    /// Hide the parser's internal details from the annotation tree.
    ///
    /// See [`TraceOpaque`] for more info.
    fn trace_opaque(self, name: impl Into<String>) -> TraceOpaque<Self> {
        TraceOpaque::new(self, name)
    }

    /// Select a parser from a tuple by index.
    ///
    /// See [`Dispatch`] for more info.
    fn dispatch<D>(self, discriminant: D) -> Dispatch<D, Self>
    where
        Self: SameParserTuple<Input>,
        D: ForwardRefGet<Value = Option<usize>>,
    {
        Dispatch::new(discriminant, self)
    }

    /// Repeat the parser with different parameter values each time.
    ///
    /// See [`Parameterize`] for more info.
    fn parameterize<V, S>(self, parameters: V, param_input: S) -> Parameterize<S, V, Self>
    where
        V: Parameters,
        S: ParameterInput<Value = V::Item>,
    {
        Parameterize::new(parameters, param_input, self)
    }

    /// Parse a value surrounded by left and right delimiters.
    ///
    /// See [`Surrounded`] for more info.
    fn surrounded_by<L, R>(self, left: L, right: R) -> Surrounded<L, Self, R>
    where
        L: Parser<Input>,
        R: Parser<Input>,
    {
        Surrounded::new(left, self, right)
    }

    /// Parse a value surrounded by the same delimiter on both sides.
    ///
    /// See [`SurroundedSymmetrical`] for more info.
    fn surrounded_by_sym<O>(self, outer: O) -> SurroundedSymmetrical<Self, O>
    where
        O: Parser<Input>,
    {
        SurroundedSymmetrical::new(self, outer)
    }

    /// Validate that the parser's output is not empty.
    ///
    /// See [`Verify`] for more info.
    fn non_empty<T>(self) -> Verify<Self, impl FnMut(&Self::Output) -> bool>
    where
        Self: Parser<Input, Output = Vec<T>>,
    {
        self.verify(|values| !values.is_empty())
    }

    /// Parse a fixed number of elements separated by a delimiter.
    ///
    /// See [`SeparatedArray`] for more info.
    fn separated_arr<const N: usize, S>(
        self,
        separator: S,
    ) -> SeparatedArray<Self, S, [Self::Output; N]>
    where
        S: Parser<Input>,
    {
        SeparatedArray::new(separator, self)
    }

    /// Parse a runtime-determined number of elements separated by a delimiter.
    ///
    /// See [`SeparatedVec`] for more info.
    fn separated_vec<S, C>(self, separator: S, count: C) -> SeparatedVec<Self, S, C>
    where
        S: Parser<Input>,
        C: ForwardRefGet,
        C::Value: AsPrimitive<usize>,
    {
        SeparatedVec::new(separator, self, count)
    }

    /// Parse a tuple of parsers with separators between them.
    ///
    /// See [`SeparatedTuple`] for more info.
    fn separated_tuple<S>(self, separator: S) -> SeparatedTuple<S, Self>
    where
        Self: ParserTuple<Input>,
        S: Parser<Input>,
    {
        SeparatedTuple::new(separator, self)
    }

    /// Parse a prefix, then return the result of the keeper parser.
    ///
    /// See [`Preceded`] for more info.
    fn ignore_then<P>(self, keep: P) -> Preceded<Self, P>
    where
        P: Parser<Input>,
    {
        Preceded::new(self, keep)
    }

    /// Parse a keeper, then a suffix, returning the keeper's result.
    ///
    /// See [`Terminated`] for more info.
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
