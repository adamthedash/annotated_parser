use crate::{Annotation, FoldResult, Parser, ParserSpec, helpers::fold_child_err};

pub struct Surrounded<L, P, R> {
    left: L,
    inner: P,
    right: R,
}

impl<L, P, R> Surrounded<L, P, R> {
    pub fn new<Input>(left: L, inner: P, right: R) -> Self
    where
        L: Parser<Input>,
        P: Parser<Input>,
        R: Parser<Input>,
    {
        Self { left, inner, right }
    }
}

impl<Input, L, P, R> Parser<Input> for Surrounded<L, P, R>
where
    L: Parser<Input>,
    P: Parser<Input>,
    R: Parser<Input>,
{
    type Output = P::Output;

    fn name(&self) -> String {
        "surrounded".to_owned()
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::new(
            self.name(),
            vec![self.left.spec(), self.inner.spec(), self.right.spec()],
        )
    }

    fn annotate(&mut self, input: &mut Input) -> crate::AnnotatedResult<Self::Output> {
        let (_left, offset, child_annotations) =
            self.left
                .annotate(input)
                .fold(vec![], 0, || self.name(), 0)?;

        let (value, offset, child_annotations) =
            self.inner
                .annotate(input)
                .fold(child_annotations, offset, || self.name(), 1)?;

        let (_right, offset, child_annotations) =
            self.right
                .annotate(input)
                .fold(child_annotations, offset, || self.name(), 2)?;

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let (_left, offset) = self
            .left
            .parse(input)
            .map_err(|annotation| fold_child_err(annotation, vec![], 0, self.name(), 0))?;

        let (value, offset) = self
            .inner
            .parse(input)
            .map_err(|annotation| fold_child_err(annotation, vec![], offset, self.name(), 1))?;

        let (_right, offset) = self
            .right
            .parse(input)
            .map_err(|annotation| fold_child_err(annotation, vec![], offset, self.name(), 2))?;

        Ok((value, offset))
    }
}

pub struct SurroundedSymmetrical<P, Q> {
    inner: P,
    outer: Q,
}

impl<P, Q> SurroundedSymmetrical<P, Q> {
    pub fn new<Input>(inner: P, outer: Q) -> Self
    where
        P: Parser<Input>,
        Q: Parser<Input>,
    {
        Self { inner, outer }
    }
}

impl<Input, P, Q> Parser<Input> for SurroundedSymmetrical<P, Q>
where
    P: Parser<Input>,
    Q: Parser<Input>,
{
    type Output = P::Output;

    fn name(&self) -> String {
        "surrounded_sym".to_owned()
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::new(self.name(), vec![self.outer.spec(), self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut Input) -> crate::AnnotatedResult<Self::Output> {
        let (_left, offset, child_annotations) =
            self.outer
                .annotate(input)
                .fold(vec![], 0, || self.name(), 0)?;

        let (value, offset, child_annotations) =
            self.inner
                .annotate(input)
                .fold(child_annotations, offset, || self.name(), 1)?;

        let (_right, offset, child_annotations) =
            self.outer
                .annotate(input)
                .fold(child_annotations, offset, || self.name(), 0)?;

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let (_left, offset) = self
            .outer
            .parse(input)
            .map_err(|annotation| fold_child_err(annotation, vec![], 0, self.name(), 0))?;

        let (value, offset) = self
            .inner
            .parse(input)
            .map_err(|annotation| fold_child_err(annotation, vec![], offset, self.name(), 1))?;

        let (_right, offset) = self
            .outer
            .parse(input)
            .map_err(|annotation| fold_child_err(annotation, vec![], offset, self.name(), 0))?;

        Ok((value, offset))
    }
}

#[cfg(test)]
mod tests {
    use crate::AnnotationResult;

    use super::*;

    #[test]
    fn test_asym_good() {
        let mut input = "\"hello\" world";
        let mut parser = Surrounded::new("\"", "hello", "\"");

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, "hello");
        assert_eq!(input, " world");
    }

    #[test]
    fn test_asym_bad() {
        let mut input = "\"hello world";
        let mut parser = Surrounded::new("\"", "hello", "\"");

        let annotation = parser.parse(&mut input).unwrap_err();
        assert!(matches!(
            annotation,
            Annotation {
                result: AnnotationResult::Child { .. },
                ..
            }
        ));

        assert_eq!(input, " world");
    }

    #[test]
    fn test_sym() {
        let mut input = "\"hello\" world";
        let mut parser = SurroundedSymmetrical::new("hello", "\"");

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, "hello");
        assert_eq!(input, " world");
    }
}
