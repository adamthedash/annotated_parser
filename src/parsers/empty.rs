use crate::{AnnotatedResult, Annotation, Parser, ParserSpec};

/// Always succeeds, consumes nothing
pub struct Empty;

impl Parser for Empty {
    type Output = ();

    fn name(&self) -> String {
        "empty".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn annotate(&mut self, _input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let annotation = Annotation::success(self.name(), 0..0, (), vec![]);
        Ok(((), annotation))
    }

    #[inline(always)]
    fn parse(&mut self, _input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        Ok(((), 0))
    }
}
