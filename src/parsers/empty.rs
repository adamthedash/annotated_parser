use crate::{AnnotatedResult, Annotation, Parser, ParserSpec};

/// Always succeeds, consumes nothing
pub struct Empty;

impl<'a> Parser<'a> for Empty {
    type Input = &'a [u8];

    type Output = ();

    fn name(&self) -> String {
        "empty".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn annotate(&mut self, _input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
        let annotation = Annotation::success(self.name(), 0..0, (), vec![]);
        Ok(((), annotation))
    }

    #[inline(always)]
    fn parse(&mut self, _input: &mut Self::Input) -> crate::ParseResult<Self::Output> {
        Ok(((), 0))
    }
}
