use std::fmt::Debug;

use super::{TakeTillExc, TakeTillInc};
use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec, helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

impl<P> Parser<&[u8]> for TakeTillExc<P>
where
    P: for<'a> Parser<&'a [u8]>,
{
    type Output = Vec<u8>;

    fn name(&self) -> String {
        "take_till_exc".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &[u8],
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

            // Advance one byte
            end += 1;
            *input = &input[1..];
        }

        let taken = original[..end].to_vec();

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..end, taken.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..end)
        };

        Ok((taken, annotation))
    }
}

impl<P, PO> Parser<&[u8]> for TakeTillInc<P>
where
    P: for<'a> Parser<&'a [u8], Output = PO>,
    PO: Clone + Debug + 'static,
{
    type Output = (Vec<u8>, PO);

    fn name(&self) -> String {
        "take_till_inc".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &[u8],
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

            // Advance one byte
            end += 1;
            *input = &input[1..];
        }

        let taken = original[..end].to_vec();
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
