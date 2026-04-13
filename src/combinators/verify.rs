use crate::{Annotation, FoldResult, Parser, ParserSpec, Result, helpers::fold_child_err};

#[derive(Clone)]
pub struct Verify<P, F> {
    inner: P,
    func: F,
}

impl<P, F> Verify<P, F>
where
    P: Parser,
    F: FnMut(&P::Output) -> bool,
{
    pub fn new(inner: P, func: F) -> Self {
        Self { inner, func }
    }
}

impl<P, F> Parser for Verify<P, F>
where
    P: Parser,
    F: FnMut(&P::Output) -> bool,
{
    type Output = P::Output;

    fn name(&self) -> String {
        "verify".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let (value, offset, child_annotations) =
            self.inner.parse(input).fold(vec![], 0, &self.name(), 0)?;

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
    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        let (value, offset) = self
            .inner
            .parse_speedy(input)
            .map_err(|a| fold_child_err(a, vec![], 0, &self.name(), 0))?;

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
