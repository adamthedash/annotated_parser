use super::value::DelayedVal;
use super::{DelayedParser, DelayedValSet};
use crate::{AnnotatedResult, Parser, ParserSpec};

/// A parser whos output can be referenced before it has been executed
pub struct Delayed<I>
where
    I: Parser,
{
    inner: I,
    /// This will be populated / overwritten whenever the parser is ran.
    value: DelayedVal<I::Output>,
}

impl<I: Parser> Delayed<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            value: DelayedVal::default(),
        }
    }
}

impl<I: Parser> Parser for Delayed<I> {
    type Output = DelayedVal<I::Output>;

    fn name(&self) -> String {
        self.inner.name()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec()
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let (out, anno) = self.inner.annotate(input)?;

        // Set the shared value
        self.value.set(out);

        Ok((self.value.clone(), anno))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let (out, offset) = self.inner.parse(input)?;

        // Set the shared value
        self.value.set(out);

        Ok((self.value.clone(), offset))
    }
}

impl<P> DelayedParser for Delayed<P>
where
    P: Parser,
{
    type Value = P::Output;
    type DelayedValue = Self::Output;

    fn output(&self) -> Self::DelayedValue {
        self.value.clone()
    }
}
