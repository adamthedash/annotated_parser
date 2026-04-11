use crate::{
    Annotation, FoldResult, Parser, ParserSpec, Result, combinators::delayed::DelayedValGet,
    helpers::fold_child_err,
};

/// A parser which may or may not be ran depending on the result of some previous parser
pub struct Cond<D, P>
where
    D: DelayedValGet,
{
    value: D,
    cond: fn(&D::Value) -> bool,
    inner: P,
}

impl<D, P> Cond<D, P>
where
    D: DelayedValGet,
{
    pub fn new(value: D, cond: fn(&D::Value) -> bool, inner: P) -> Self {
        Self { value, cond, inner }
    }
}

impl<D, P> Parser for Cond<D, P>
where
    D: DelayedValGet,
    P: Parser,
{
    type Output = Option<P::Output>;

    fn name(&self) -> String {
        "cond".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let value = self.value.get();

        let (value, span, child_annotations) = if (self.cond)(&value) {
            let (value, span, child_annotations) =
                self.inner.parse(input).fold(vec![], 0, &self.name(), 0)?;

            (Some(value), span, child_annotations)
        } else {
            (None, 0..0, vec![])
        };

        let annotation = Annotation::success(&self.name(), span, &value, child_annotations);

        Ok((value, annotation))
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        let value = self.value.get();

        if !(self.cond)(&value) {
            return Ok((None, 0));
        }

        let (value, offset) = self
            .inner
            .parse_speedy(input)
            .map_err(|a| fold_child_err(a, vec![], 0, &self.name(), 0))?;

        Ok((Some(value), offset))
    }
}
