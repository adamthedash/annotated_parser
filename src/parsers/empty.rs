use crate::{AnnotatedResult, Annotation, Parser, ParserSpec};

/// Always succeeds, consumes nothing
pub struct Empty;

impl<Input> Parser<Input> for Empty {
    type Output = ();

    fn name(&self) -> String {
        "empty".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<Input>::name(self))
    }

    fn annotate(&mut self, _input: &mut Input) -> AnnotatedResult<Self::Output> {
        let annotation = Annotation::success(Parser::<Input>::name(self), 0..0, (), vec![]);
        Ok(((), annotation))
    }

    #[inline(always)]
    fn parse(&mut self, _input: &mut Input) -> crate::ParseResult<Self::Output> {
        Ok(((), 0))
    }
}
