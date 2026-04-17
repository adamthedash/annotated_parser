use crate::{
    AnnotatedResult, Annotation, FoldAnnotatedResult, Parser, ParserSpec, combinators::Checkpoint,
};

/// Optional parser. If inner parser fails, then this succeed but produces no value
pub struct Opt<P> {
    inner: Checkpoint<P>,
}

impl<P> Opt<P> {
    pub fn new<Input>(inner: P) -> Self
    where
        P: Parser<Input>,
        Input: Copy,
    {
        Self {
            inner: Checkpoint::new(inner),
        }
    }
}

impl<Input, P> Parser<Input> for Opt<P>
where
    P: Parser<Input>,
    Input: Copy,
{
    type Output = Option<P::Output>;

    fn name(&self) -> String {
        "opt".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        let res = self
            .inner
            .annotate(input)
            .fold(vec![], 0, || self.name(), 0);

        let (out, offset, child_annotations) = match res {
            Ok((out, offset, child_annotations)) => (Some(out), offset, child_annotations),
            // TODO: Should we be passing up child annotations here?
            Err(child_annotation) => (None, 0, vec![child_annotation]),
        };

        let annotation =
            Annotation::success(self.name(), 0..offset, out.clone(), child_annotations);

        Ok((out, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let Ok((value, offset)) = self.inner.parse(input) else {
            return Ok((None, 0));
        };

        Ok((Some(value), offset))
    }
}

#[cfg(test)]
mod tests {
    use crate::{ByteParser, Parser, ParserAdapter};
    #[test]
    fn test_some() {
        let mut parser = u32::LE.optional();
        let mut input = [0; 5].as_slice();

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, Some(0));
        assert_eq!(input, &[0]);
    }

    #[test]
    fn test_none_incomplete() {
        let mut parser = u32::LE.optional();
        let mut input = [0; 3].as_slice();

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, None);
        assert_eq!(input, &[0; 3]);
    }

    #[test]
    fn test_none_invalid() {
        let mut parser = u32::LE.verify(|x| *x == 1).optional();
        let mut input = [0; 5].as_slice();

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, None);
        assert_eq!(input, &[0; 5]);
    }
}
