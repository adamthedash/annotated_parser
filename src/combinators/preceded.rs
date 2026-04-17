use crate::{
    Annotation, AnnotationReturn, Parser, ParserSpec, helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

pub struct Preceded<I, K> {
    ignore: I,
    keep: K,
}

impl<I, K> Preceded<I, K> {
    pub fn new<Input>(ignore: I, keep: K) -> Self
    where
        I: Parser<Input>,
        K: Parser<Input>,
    {
        Self { ignore, keep }
    }
}

impl<Input, I, K> Parser<Input> for Preceded<I, K>
where
    I: Parser<Input>,
    K: Parser<Input>,
{
    type Output = K::Output;

    fn name(&self) -> String {
        "preceded".to_owned()
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::new(self.name(), vec![self.ignore.spec(), self.keep.spec()])
    }

    #[inline(always)]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let mut child_annotations = annotation_mode.success.then(Vec::new);
        let mut offset = 0;

        (_, offset, child_annotations) = self.ignore.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            offset,
            0,
        )?;

        let value;
        (value, offset, child_annotations) = self.keep.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            offset,
            1,
        )?;

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
    use crate::Parser;

    use super::Preceded;

    #[test]
    fn test() {
        let mut input = "hello_world";
        let mut parser = Preceded::new("hello", "_world");

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, "_world");
        assert_eq!(input, "");
    }
}
