use crate::{
    Annotation, AnnotationMode, AnnotationReturn, ParseWithResult, Parser, ParserInfo, ParserSpec,
};

/// A parser that succeeds only when the input has been fully consumed.
///
/// Fails if any data remains. Works with both `&[u8]` and `&str` inputs.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::EoF;
///
/// let mut input = "";
/// let (_, _) = EoF.parse(&mut input).unwrap();
/// ```
pub struct EoF;

impl ParserInfo for EoF {
    fn name(&self) -> String {
        "eof".to_string()
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::empty(self.name())
    }
}

impl Parser<&[u8]> for EoF {
    type Output = ();

    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        if !input.is_empty() {
            let annotation = if annotation_mode.fail {
                Annotation::invalid(self.name(), 0..0, "Data remaining".to_owned(), vec![]).into()
            } else {
                AnnotationReturn::Span(0..0)
            };

            return Err(annotation);
        }

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..0, (), vec![]).into()
        } else {
            AnnotationReturn::Span(0..0)
        };

        Ok(((), annotation))
    }
}

impl Parser<&str> for EoF {
    type Output = ();

    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        if !input.is_empty() {
            let annotation = if annotation_mode.fail {
                Annotation::invalid(self.name(), 0..0, "Data remaining".to_owned(), vec![]).into()
            } else {
                AnnotationReturn::Span(0..0)
            };

            return Err(annotation);
        }

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..0, (), vec![]).into()
        } else {
            AnnotationReturn::Span(0..0)
        };

        Ok(((), annotation))
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::*;

    #[test]
    fn test_empty() {
        let mut parser = EoF;
        let mut input = "";
        let (_, annotation) = parser.parse_with(&mut input, AnnotationMode::NONE).unwrap();
        assert!(matches!(
            annotation,
            AnnotationReturn::Span(Range { start: 0, end: 0 })
        ));
    }

    #[test]
    fn test_not_empty() {
        let mut parser = EoF;
        let mut input = "abc";
        let annotation = parser
            .parse_with(&mut input, AnnotationMode::NONE)
            .unwrap_err();
        assert!(matches!(
            annotation,
            AnnotationReturn::Span(Range { start: 0, end: 0 })
        ));
    }
}
