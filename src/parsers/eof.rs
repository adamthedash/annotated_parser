use crate::{Annotation, AnnotationMode, AnnotationReturn, ParseWithResult, Parser, ParserSpec};

pub struct EoF;

impl Parser<&[u8]> for EoF {
    type Output = ();

    fn name(&self) -> String {
        "eof".to_string()
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::empty(Parser::<&[u8]>::name(self))
    }

    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        if !input.is_empty() {
            let annotation = if annotation_mode.fail {
                Annotation::invalid(
                    Parser::<&[u8]>::name(self),
                    0..0,
                    "Data remaining".to_owned(),
                    vec![],
                )
                .into()
            } else {
                AnnotationReturn::Span(0..0)
            };

            return Err(annotation);
        }

        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<&[u8]>::name(self), 0..0, (), vec![]).into()
        } else {
            AnnotationReturn::Span(0..0)
        };

        Ok(((), annotation))
    }
}

impl Parser<&str> for EoF {
    type Output = ();

    fn name(&self) -> String {
        "eof".to_string()
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::empty(Parser::<&str>::name(self))
    }

    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        if !input.is_empty() {
            let annotation = if annotation_mode.fail {
                Annotation::invalid(
                    Parser::<&str>::name(self),
                    0..0,
                    "Data remaining".to_owned(),
                    vec![],
                )
                .into()
            } else {
                AnnotationReturn::Span(0..0)
            };

            return Err(annotation);
        }

        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<&str>::name(self), 0..0, (), vec![]).into()
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
