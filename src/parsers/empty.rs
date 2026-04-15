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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParserAdapter;

    #[test]
    fn test_bare() {
        let mut parser = Empty;

        let input = vec![0_u8; 4];
        let (_value, _) = parser.parse(&mut input.as_slice()).unwrap();
    }

    #[test]
    fn test_combinator() {
        /// This ensures we have correct type inference when Empty is used in a combinator
        fn empty() -> impl for<'a> Parser<&'a [u8]> {
            Empty
        }

        let mut parser = empty().repeat::<2>();

        let input = vec![0_u8; 4];
        let (_value, _) = parser.parse(&mut input.as_slice()).unwrap();
    }
}
