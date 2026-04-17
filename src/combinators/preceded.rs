use crate::{Annotation, FoldAnnotatedResult, Parser, ParserSpec, helpers::FoldParseResult};

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

    fn annotate(&mut self, input: &mut Input) -> crate::AnnotatedResult<Self::Output> {
        let (_before, offset, child_annotations) =
            self.ignore
                .annotate(input)
                .fold(vec![], 0, || self.name(), 0)?;

        let (value, offset, child_annotations) =
            self.keep
                .annotate(input)
                .fold(child_annotations, offset, || self.name(), 1)?;

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let (_before, offset) = self
            .ignore
            .parse(input).fold(0, || self.name(), 0)?;

        let (value, offset) = self
            .keep
            .parse(input).fold(offset, || self.name(), 1)?;

        Ok((value, offset))
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
