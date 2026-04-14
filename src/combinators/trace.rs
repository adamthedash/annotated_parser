use crate::{AnnotatedResult, Parser, ParserSpec, combinators::delayed::DelayedParser};

/// For adding a user-friendly name to the spec
#[derive(Clone)]
pub struct Trace<P> {
    inner: P,
    name: String,
}

impl<P: Parser> Trace<P> {
    pub fn new(inner: P, name: impl Into<String>) -> Self {
        Self {
            inner,
            name: name.into(),
        }
    }
}

impl<P: Parser> Parser for Trace<P> {
    type Output = P::Output;

    fn name(&self) -> String {
        // TODO: Pass through inner name?
        //  Or "trace"?
        //  Or self.name?
        self.name.clone()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec().with_friendly(self.name())
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        self.inner.annotate(input)
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        self.inner.parse(input)
    }
}

impl<P> DelayedParser for Trace<P>
where
    P: DelayedParser,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
        self.inner.output()
    }
}
