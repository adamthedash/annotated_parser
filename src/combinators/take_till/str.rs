use std::fmt::Debug;

use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec, helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

use super::{TakeTillExc, TakeTillInc};

impl<P> Parser<&str> for TakeTillExc<P>
where
    P: for<'a> Parser<&'a str>,
{
    type Output = String;

    fn name(&self) -> String {
        "take_till_exc".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        // PERF: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.parse_with(input, AnnotationMode::NONE).is_err() {
            if end == original.len() {
                // EoF
                let annotation = if annotation_mode.fail {
                    Annotation::incomplete(self.name(), 0, vec![]).into()
                } else {
                    AnnotationReturn::Start(0)
                };

                return Err(annotation);
            }

            // Advance one char
            end += input
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(input.len());

            *input = &original[end..];
        }

        let taken = original[..end].to_string();
        let taken_chars = taken.chars().count();

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..taken_chars, taken.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..taken_chars)
        };

        Ok((taken, annotation))
    }
}

impl<P, PO> Parser<&str> for TakeTillInc<P>
where
    P: for<'a> Parser<&'a str, Output = PO>,
    PO: Clone + Debug + 'static,
{
    type Output = (String, PO);

    fn name(&self) -> String {
        "take_till_inc".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        let inner_mode = AnnotationMode {
            success: annotation_mode.success,
            fail: false,
        };

        let mut child_annotations = annotation_mode.success.then(Vec::new);
        let mut offset = 0;

        let value;
        loop {
            let res = self.inner.parse_with(input, inner_mode);
            if res.is_ok() {
                // Terminator found
                (value, offset, child_annotations) = res
                    .fold(
                        annotation_mode,
                        || self.name(),
                        child_annotations,
                        offset,
                        0,
                    )
                    .expect("Happy path");

                break;
            }

            if end == original.len() {
                // EoF
                let annotation = if annotation_mode.fail {
                    Annotation::incomplete(self.name(), 0, child_annotations.unwrap()).into()
                } else {
                    AnnotationReturn::Start(0)
                };

                return Err(annotation);
            }

            // Advance one char
            end += input
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(input.len());

            *input = &original[end..];
        }

        let taken = original[..end].to_string();
        let out = (taken, value);

        let annotation = if annotation_mode.success {
            Annotation::success(
                self.name(),
                0..offset,
                out.clone(),
                child_annotations.unwrap(),
            )
            .into()
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok((out, annotation))
    }
}
