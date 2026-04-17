use std::fmt::Debug;

use crate::{
    AnnotatedResult, Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec,
    helpers::fold_success, parser::ParseWithResult,
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
                    AnnotationReturn::Annotated(Annotation::incomplete(self.name(), 0, vec![]))
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
            AnnotationReturn::Annotated(Annotation::success(
                self.name(),
                0..taken_chars,
                taken.clone(),
                vec![],
            ))
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

    fn annotate(&mut self, input: &mut &str) -> AnnotatedResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        let (value, offset, child_annotations) = loop {
            if let Ok((value, annotation)) = self.inner.annotate(input) {
                let (offset, child_annotations) = fold_success(annotation, vec![], end, 0);
                break (value, offset, child_annotations);
            }

            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one char
            end += input
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(input.len());

            *input = &original[end..];
        };

        let taken = original[..end].to_string();

        let annotation = Annotation::success(
            self.name(),
            0..offset,
            (taken.clone(), value.clone()),
            child_annotations,
        );

        Ok(((taken, value), annotation))
    }

    fn parse(&mut self, input: &mut &str) -> crate::ParseResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        let (value, offset) = loop {
            if let Ok((value, offset)) = self.inner.parse_with(input, AnnotationMode::NONE) {
                let AnnotationReturn::Span(span) = offset else {
                    unreachable!();
                };
                break (value, span.end);
            }

            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one char
            end += input
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(input.len());

            *input = &original[end..];
        };

        let taken = original[..end].to_string();

        Ok(((taken, value), offset))
    }
}
