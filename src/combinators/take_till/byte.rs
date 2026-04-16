use std::fmt::Debug;

use super::{TakeTillExc, TakeTillInc};
use crate::{AnnotatedResult, Annotation, Parser, ParserSpec, helpers::fold_success};

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

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        // PERF: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.annotate(input).is_err() {
            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one byte
            end += 1;
            *input = &input[1..];
        }

        let bytes = original[..end].to_vec();

        let annotation = Annotation::success(self.name(), 0..bytes.len(), bytes.clone(), vec![]);

        Ok((bytes, annotation))
    }

    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        // PERF: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.annotate(input).is_err() {
            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one byte
            end += 1;
            *input = &input[1..];
        }

        Ok((original[..end].to_vec(), end))
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

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
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

            // Advance one byte
            end += 1;
            *input = &input[1..];
        };

        let bytes = original[..end].to_vec();

        let annotation = Annotation::success(
            self.name(),
            0..offset,
            (bytes.clone(), value.clone()),
            child_annotations,
        );

        Ok(((bytes, value), annotation))
    }

    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        let (value, offset) = loop {
            if let Ok((value, offset)) = self.inner.parse(input) {
                break (value, offset);
            }

            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one byte
            end += 1;
            *input = &input[1..];
        };

        let bytes = original[..end].to_vec();

        Ok(((bytes, value), offset))
    }
}
