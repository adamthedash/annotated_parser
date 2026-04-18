use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec, combinators::Checkpoint,
    helpers::FoldParseWithResult, parser::ParseWithResult,
};

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

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let mut child_annotations = annotation_mode.success.then(Vec::new);

        let mut values = vec![];
        let mut offset = 0;

        let inner_mode = AnnotationMode {
            success: annotation_mode.success,
            fail: false,
        };

        let mut value;
        loop {
            let res = self.inner.parse_with(input, inner_mode);
            if res.is_err() {
                break;
            }

            (value, offset, child_annotations) = res
                .fold(inner_mode, || self.name(), child_annotations, offset, 0)
                .expect("In happy path");

            values.push(value);
        }

        let annotation = if annotation_mode.success {
            Annotation::success(
                self.name(),
                0..offset,
                values.clone(),
                child_annotations.unwrap(),
            )
            .into()
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok((values, annotation))
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
