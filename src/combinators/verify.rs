use crate::{
    Annotation, AnnotationReturn, Parser, ParserSpec, helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

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

    #[inline(always)]
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
                AnnotationReturn::Annotated(Annotation::invalid(
                    self.name(),
                    0..offset,
                    "Validation failure".to_owned(),
                    child_annotations.take().unwrap_or_default(),
                ))
            } else {
                AnnotationReturn::Span(0..offset)
            };

            return Err(annotation);
        }

        let annotation = if annotation_mode.success {
            AnnotationReturn::Annotated(Annotation::success(
                self.name(),
                0..offset,
                value.clone(),
                child_annotations.take().unwrap(),
            ))
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok((value, annotation))
    }
}
