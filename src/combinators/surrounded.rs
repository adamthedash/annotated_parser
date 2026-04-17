use crate::{
    Annotation, AnnotationReturn, Parser, ParserSpec, helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

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

    #[inline(always)]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let mut child_annotations = annotation_mode.success.then(Vec::new);
        let mut offset = 0;

        (_, offset, child_annotations) = self.left.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            offset,
            0,
        )?;

        let value;
        (value, offset, child_annotations) = self.inner.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            offset,
            1,
        )?;

        (_, offset, child_annotations) = self.right.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            offset,
            2,
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

    #[inline(always)]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let mut child_annotations = annotation_mode.success.then(Vec::new);
        let mut offset = 0;

        (_, offset, child_annotations) = self.outer.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            offset,
            0,
        )?;

        let value;
        (value, offset, child_annotations) = self.inner.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            offset,
            1,
        )?;

        (_, offset, child_annotations) = self.outer.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            offset,
            0,
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
