use crate::{AnnotatedResult, Parser, ParserSpec, combinators::delayed::DelayedParser};

/// For adding a user-friendly name to the spec
#[derive(Clone)]
pub struct Trace<P> {
    inner: P,
    name: String,
}

impl<P> Trace<P> {
    pub fn new<Input>(inner: P, name: impl Into<String>) -> Self
    where
        P: Parser<Input>,
    {
        Self {
            inner,
            name: name.into(),
        }
    }
}

impl<Input, P> Parser<Input> for Trace<P>
where
    P: Parser<Input>,
{
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        self.inner.annotate(input)
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        self.inner.parse(input)
    }
}

impl<Input, P> DelayedParser<Input> for Trace<P>
where
    P: DelayedParser<Input>,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
        self.inner.output()
    }
}
