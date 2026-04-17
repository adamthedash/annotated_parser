use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec,
    combinators::delayed::DelayedValGet, helpers::FoldParseWithResult, parser::ParseWithResult,
};

/// A parser which may or may not be ran depending on the result of some previous parser
pub struct Cond<C, P> {
    cond: C,
    inner: P,
}

impl<C, P> Cond<C, P>
where
    C: DelayedValGet<Value = bool>,
{
    pub fn new<Input>(cond: C, inner: P) -> Self
    where
        P: Parser<Input>,
    {
        Self { cond, inner }
    }
}

impl<Input, C, P> Parser<Input> for Cond<C, P>
where
    C: DelayedValGet<Value = bool>,
    P: Parser<Input>,
{
    type Output = Option<P::Output>;

    fn name(&self) -> String {
        "cond".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline(always)]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let mut child_annotations = annotation_mode.success.then(Vec::new);
        let mut offset = 0;
        let mut value = None;

        if *self.cond.get() {
            let out;
            (out, offset, child_annotations) = self.inner.parse_with(input, annotation_mode).fold(
                annotation_mode,
                || self.name(),
                child_annotations,
                offset,
                0,
            )?;

            value = Some(out);
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
