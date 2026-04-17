use crate::{Annotation, FoldAnnotatedResult, Parser, ParserSpec, helpers::FoldParseResult};

pub struct Terminated<I, K> {
    ignore: I,
    keep: K,
}

impl<I, K> Terminated<I, K> {
    pub fn new<Input>(keep: K, ignore: I) -> Self
    where
        I: Parser<Input>,
        K: Parser<Input>,
    {
        Self { ignore, keep }
    }
}

impl<Input, I, K> Parser<Input> for Terminated<I, K>
where
    I: Parser<Input>,
    K: Parser<Input>,
{
    type Output = K::Output;

    fn name(&self) -> String {
        "terminated".to_owned()
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::new(self.name(), vec![self.keep.spec(), self.ignore.spec()])
    }

    fn annotate(&mut self, input: &mut Input) -> crate::AnnotatedResult<Self::Output> {
        let (value, offset, child_annotations) =
            self.keep
                .annotate(input)
                .fold(vec![], 0, || self.name(), 0)?;

        let (_after, offset, child_annotations) =
            self.ignore
                .annotate(input)
                .fold(child_annotations, offset, || self.name(), 1)?;

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let (value, offset) = self
            .keep
            .parse(input).fold(0, || self.name(), 0)?;

        let (_after, offset) = self
            .ignore
            .parse(input).fold(offset, || self.name(), 1)?;

        Ok((value, offset))
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;

    use super::Terminated;

    #[test]
    fn test() {
        let mut input = "hello_world";
        let mut parser = Terminated::new("hello", "_world");

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, "hello");
        assert_eq!(input, "");
    }
}
