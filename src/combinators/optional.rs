use crate::{Annotation, FoldResult, Parser, ParserSpec, Result};

/// Optional parser. If inner parser fails, then this succeed but produces no value
pub struct Opt<I>(pub I);

impl<I> Parser for Opt<I>
where
    I: Parser,
{
    type Output = Option<I::Output>;

    fn name(&self) -> String {
        "opt".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.0.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let res = self.0.parse(input).fold(vec![], 0, &self.name(), 0);

        let (out, span, child_annotations) = match res {
            Ok((out, span, child_annotations)) => (Some(out), span, child_annotations),
            // TODO: Should we be passing up child annotations here?
            Err(child_annotation) => (None, 0..0, vec![child_annotation]),
        };

        let annotation = Annotation::success(&self.name(), span, &out, child_annotations);

        Ok((out, annotation))
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        let Ok((value, offset)) = self.0.parse_speedy(input) else {
            return Ok((None, 0));
        };

        Ok((Some(value), offset))
    }
}
