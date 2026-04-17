use crate::{
    AnnotatedResult, Annotation, FoldAnnotatedResult, Parser, ParserSpec, helpers::FoldParseResult,
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        let (value, offset, child_annotations) =
            self.inner
                .annotate(input)
                .fold(vec![], 0, || self.name(), 0)?;

        if !(self.func)(&value) {
            return Err(Annotation::invalid(
                self.name(),
                0..offset,
                "Validation failure".to_owned(),
                child_annotations,
            ));
        }

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let (value, offset) = self
            .inner
            .parse(input).fold(0, || self.name(), 0)?;

        if !(self.func)(&value) {
            return Err(Annotation::invalid(
                self.name(),
                0..offset,
                "Validation failure".to_owned(),
                vec![],
            ));
        }

        Ok((value, offset))
    }
}
