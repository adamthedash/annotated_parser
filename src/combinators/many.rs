use crate::{Annotation, Parser, ParserSpec, combinators::Checkpoint, helpers::fold_success};

/// Apply the inner parser repeatedly until it fails
pub struct Many<P> {
    inner: Checkpoint<P>,
}

impl<P> Many<P> {
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

impl<Input, P> Parser<Input> for Many<P>
where
    P: Parser<Input>,
    Input: Copy,
{
    type Output = Vec<P::Output>;

    fn name(&self) -> String {
        "many".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut Input) -> crate::AnnotatedResult<Self::Output> {
        let mut values = vec![];
        let mut child_annotations = vec![];
        let mut offset = 0;

        while let Ok((value, annotation)) = self.inner.annotate(input) {
            (offset, child_annotations) = fold_success(annotation, child_annotations, offset, 0);

            values.push(value);
        }

        let annotation =
            Annotation::success(self.name(), 0..offset, values.clone(), child_annotations);

        Ok((values, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let mut values = vec![];
        let mut offset = 0;

        // PERF: This will always allocate one annotation when the inner parser fails. Any way to
        // avoid?
        while let Ok((value, taken)) = self.inner.parse(input) {
            offset += taken;
            values.push(value);
        }

        Ok((values, offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_early() {
        let mut input = b"aaaabb".as_slice();
        let mut parser = Many::new(b"a");

        let (value, offset) = parser.parse(&mut input).unwrap();

        assert_eq!(value, vec![b"a"; 4]);
        assert_eq!(input, b"bb");
        assert_eq!(offset, 4);
    }

    #[test]
    fn test_end() {
        let mut input = b"aaaa".as_slice();
        let mut parser = Many::new(b"a");

        let (value, offset) = parser.parse(&mut input).unwrap();

        assert_eq!(value, vec![b"a"; 4]);
        assert_eq!(input, b"");
        assert_eq!(offset, 4);
    }

    #[test]
    fn test_none() {
        let mut input = b"bb".as_slice();
        let mut parser = Many::new(b"a");

        let (value, offset) = parser.parse(&mut input).unwrap();

        assert_eq!(value, vec![b"a"; 0]);
        assert_eq!(input, b"bb");
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_emty() {
        let mut input = b"".as_slice();
        let mut parser = Many::new(b"a");

        let (value, offset) = parser.parse(&mut input).unwrap();

        assert_eq!(value, vec![b"a"; 0]);
        assert_eq!(input, b"");
        assert_eq!(offset, 0);
    }
}
