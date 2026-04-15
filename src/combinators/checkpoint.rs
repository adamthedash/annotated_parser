use crate::{AnnotatedResult, Parser, ParserSpec, combinators::delayed::DelayedParser};

/// Wrapper which resets the input stream on failure
pub struct Checkpoint<P>(P);

impl<P> Checkpoint<P> {
    pub fn new<Input>(inner: P) -> Self
    where
        P: Parser<Input>,
        Input: Copy,
    {
        Self(inner)
    }
}

impl<Input, P> Parser<Input> for Checkpoint<P>
where
    P: Parser<Input>,
    Input: Copy,
{
    type Output = P::Output;

    fn name(&self) -> String {
        self.0.name()
    }

    fn spec(&self) -> ParserSpec {
        self.0.spec()
    }

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        let res = self.0.annotate(input);
        if res.is_err() {
            // Reset input
            *input = checkpoint;
        }

        res
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        let res = self.0.parse(input);
        if res.is_err() {
            // Reset input
            *input = checkpoint;
        }

        res
    }
}

impl<Input, P> DelayedParser<Input> for Checkpoint<P>
where
    P: DelayedParser<Input>,
    Input: Copy,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
        self.0.output()
    }
}

/// Wrapper which resets the input stream in all cases
pub struct Peek<P>(P);

impl<P> Peek<P> {
    pub fn new<Input>(inner: P) -> Self
    where
        P: Parser<Input>,
        Input: Copy,
    {
        Self(inner)
    }
}

impl<Input, P> Parser<Input> for Peek<P>
where
    P: Parser<Input>,
    Input: Copy,
{
    type Output = P::Output;

    fn name(&self) -> String {
        self.0.name()
    }

    fn spec(&self) -> ParserSpec {
        self.0.spec()
    }

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        // TODO: On success this will return an annotation in the "future", so it might conflict
        // with follow-on annotations. Maybe return 0-span annotation instead?
        let res = self.0.annotate(input);

        // Reset input
        *input = checkpoint;

        res
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        let res = self.0.parse(input);

        // Reset input
        *input = checkpoint;

        res
    }
}

impl<Input, P> DelayedParser<Input> for Peek<P>
where
    P: DelayedParser<Input>,
    Input: Copy,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
        self.0.output()
    }
}
