use crate::{
    Annotation, AnnotationReturn, Parser, ParserSpec, helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

/// Validate the output of a parser with a predicate.
///
/// Runs the inner parser, then applies a predicate to the result.
/// If the predicate returns `false`, the parser fails with a validation error.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::ByteParser;
///
/// let mut parser = u8::LE.verify(|x| *x == 1);
/// let mut input = &[1_u8][..];
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, 1);
/// ```
#[derive(Clone)]
pub struct Verify<P, F> {
    inner: P,
    func: F,
}

impl<P, F> Verify<P, F> {
    pub fn new<Input>(inner: P, func: F) -> Self
    where
        P: Parser<Input>,
        F: FnMut(&P::Output) -> bool,
    {
        Self { inner, func }
    }
}

impl<Input, P, F> Parser<Input> for Verify<P, F>
where
    P: Parser<Input>,
    F: FnMut(&P::Output) -> bool,
{
    type Output = P::Output;

    fn name(&self) -> String {
        "verify".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        // PERF: Only allocates if we need it
        let mut child_annotations = annotation_mode.success.then(Vec::new);

        let (value, offset);
        (value, offset, child_annotations) = self.inner.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            0,
            0,
        )?;

        if !(self.func)(&value) {
            let annotation = if annotation_mode.fail {
                Annotation::invalid(
                    self.name(),
                    0..offset,
                    "Validation failure".to_owned(),
                    child_annotations.unwrap_or_default(),
                )
                .into()
            } else {
                AnnotationReturn::Span(0..offset)
            };

            return Err(annotation);
        }

        let annotation = if annotation_mode.success {
            Annotation::success(
                self.name(),
                0..offset,
                value.clone(),
                child_annotations.unwrap(),
            )
            .into()
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok((value, annotation))
    }
}
