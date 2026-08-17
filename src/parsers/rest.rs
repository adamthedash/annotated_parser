use crate::{AnnotationReturn, ParseWithResult};

use crate::{Annotation, Parser, ParserSpec};

pub struct Rest;

impl Parser<&[u8]> for Rest {
    type Output = Vec<u8>;

    fn name(&self) -> String {
        "rest".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<&str>::name(self))
    }

    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let value = input.to_owned();

        *input = &[];

        let annotation = if annotation_mode.success {
            Annotation::success(
                Parser::<&str>::name(self),
                0..value.len(),
                value.clone(),
                vec![],
            )
            .into()
        } else {
            AnnotationReturn::Span(0..value.len())
        };

        Ok((value, annotation))
    }
}

impl Parser<&str> for Rest {
    type Output = String;

    fn name(&self) -> String {
        "rest".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<&str>::name(self))
    }

    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let value = input.to_owned();
        let num_chars = value.chars().count();

        *input = "";

        let annotation = if annotation_mode.success {
            Annotation::success(
                Parser::<&str>::name(self),
                0..num_chars,
                value.clone(),
                vec![],
            )
            .into()
        } else {
            AnnotationReturn::Span(0..num_chars)
        };

        Ok((value, annotation))
    }
}
