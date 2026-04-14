use crate::{AnnotatedResult, Parser, ParserSpec, combinators::delayed::DelayedParser};

/// For adding a user-friendly name to the spec
#[derive(Clone)]
pub struct Trace<P> {
    inner: P,
    name: String,
}

impl<'a, P> Trace<P>
where
    P: Parser<'a>,
{
    pub fn new(inner: P, name: impl Into<String>) -> Self {
        Self {
            inner,
            name: name.into(),
        }
    }
}

impl<'a, P> Parser<'a> for Trace<P>
where
    P: Parser<'a>,
{
    type Input = P::Input;
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

    fn annotate(&mut self, input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
        self.inner.annotate(input)
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut Self::Input) -> crate::ParseResult<Self::Output> {
        self.inner.parse(input)
    }
}

impl<'a, P> DelayedParser<'a> for Trace<P>
where
    P: DelayedParser<'a>,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
        self.inner.output()
    }
}
