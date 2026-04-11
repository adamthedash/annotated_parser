use crate::{Parser, ParserSpec, Result};

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

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        self.inner.parse(input)
    }

    #[inline(always)]
    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        self.inner.parse_speedy(input)
    }
}
