use crate::{AnnotatedResult, Annotation, FoldResult, Parser, ParserSpec, helpers::fold_child_err};

#[derive(Clone)]
pub struct Verify<P, F> {
    inner: P,
    func: F,
}

impl<'a, P, F> Verify<P, F>
where
    P: Parser<'a>,
    F: FnMut(&P::Output) -> bool,
{
    pub fn new(inner: P, func: F) -> Self {
        Self { inner, func }
    }
}

impl<'a, P, F> Parser<'a> for Verify<P, F>
where
    P: Parser<'a>,
    F: FnMut(&P::Output) -> bool,
{
    type Input = P::Input;

    type Output = P::Output;

    fn name(&self) -> String {
        "verify".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
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
    fn parse(&mut self, input: &mut Self::Input) -> crate::ParseResult<Self::Output> {
        let (value, offset) = self
            .inner
            .parse(input)
            .map_err(|a| fold_child_err(a, vec![], 0, self.name(), 0))?;

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
