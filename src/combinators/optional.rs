use crate::parser::ParseWithResult;
use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec, combinators::Checkpoint,
    helpers::FoldParseWithResult,
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

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let inner_mode = AnnotationMode {
            success: annotation_mode.success,
            // This parser always succeeds, so we don't care about the inner failure details
            fail: false,
        };

        let mut child_annotations = annotation_mode.success.then(Vec::new);

        let res = self.inner.parse_with(input, inner_mode);

        let (value, offset) = if res.is_ok() {
            let (value, offset);
            (value, offset, child_annotations) = res
                .fold(inner_mode, || self.name(), child_annotations, 0, 0)
                .expect("In happy path");

            (Some(value), offset)
        } else {
            (None, 0)
        };

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
