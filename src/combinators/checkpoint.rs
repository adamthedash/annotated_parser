use crate::{Parser, ParserSpec, Result, combinators::delayed::DelayedParser};

/// Wrapper which resets the input stream on failure
pub struct Checkpoint<P>(pub P);

impl<P: Parser> Parser for Checkpoint<P> {
    type Output = P::Output;

    fn name(&self) -> String {
        self.0.name()
    }

    fn spec(&self) -> ParserSpec {
        self.0.spec()
    }

    fn annotate(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
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
    fn parse(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
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

impl<P> DelayedParser for Checkpoint<P>
where
    P: DelayedParser,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
        self.0.output()
    }
}

/// Wrapper which resets the input stream in all cases
pub struct Peek<P>(pub P);

impl<P: Parser> Parser for Peek<P> {
    type Output = P::Output;

    fn name(&self) -> String {
        self.0.name()
    }

    fn spec(&self) -> ParserSpec {
        self.0.spec()
    }

    fn annotate(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        // TODO: On success this will return an annotation in the "future", so it might conflict
        // with follow-on annotations. Maybe return 0-span annotation instead?
        let res = self.0.annotate(input);

        // Reset input
        *input = checkpoint;

        res
    }

    fn parse(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        let res = self.0.parse(input);

        // Reset input
        *input = checkpoint;

        res
    }
}

impl<P> DelayedParser for Peek<P>
where
    P: DelayedParser,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
        self.0.output()
    }
}
