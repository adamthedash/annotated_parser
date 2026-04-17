use crate::{
    AnnotatedResult, Annotation, FoldAnnotatedResult, Parser, ParserSpec,
    combinators::delayed::DelayedValGet, helpers::FoldParseResult,
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        let (value, offset, child_annotations) = if *self.cond.get() {
            let (value, offset, child_annotations) =
                self.inner
                    .annotate(input)
                    .fold(vec![], 0, || self.name(), 0)?;

            (Some(value), offset, child_annotations)
        } else {
            (None, 0, vec![])
        };

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        if !*self.cond.get() {
            return Ok((None, 0));
        }

        let (value, offset) = self.inner.parse(input).fold(0, || self.name(), 0)?;
        // .map_err(|a| fold_child_err(a, vec![], 0, self.name(), 0))?;

        Ok((Some(value), offset))
    }
}
