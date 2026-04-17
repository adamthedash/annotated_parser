use crate::{Annotation, AnnotationReturn, Parser, ParserSpec, parser::ParseWithResult};

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

    #[inline(always)]
    fn parse_with(
        &mut self,
        _input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<Input>::name(self), 0..0, (), vec![]).into()
        } else {
            AnnotationReturn::Span(0..0)
        };
        Ok(((), annotation))
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
