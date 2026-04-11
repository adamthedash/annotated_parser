use crate::{Annotation, Parser, ParserSpec, Result};

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

    fn parse(&mut self, _input: &mut &[u8]) -> Result<Self::Output> {
        let annotation = Annotation::success(&self.name(), 0..0, (), vec![]);
        Ok(((), annotation))
    }

    #[inline(always)]
    fn parse_speedy(&mut self, _input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        Ok(((), 0))
    }
}
