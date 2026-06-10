use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec,
    combinators::store::ForwardRefGet, helpers::FoldParseWithResult, parser::ParseWithResult,
};

/// Conditionally run a parser based on a boolean value.
///
/// If the condition is `true`, applies the inner parser and returns `Some(output)`.
/// If the condition is `false`, returns `None` without consuming input.
/// The condition is evaluated via a `ForwardRefGet<bool>`.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::combinators::Cond;
/// use annotated_parser::ForwardRef;
///
/// let flag = ForwardRef::with_value(true);
/// let mut parser = Cond::new(flag, b"a");
/// let mut input = b"aaa".as_slice();
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, Some(b"a"));
/// assert_eq!(input, b"aa");
/// ```
pub struct Cond<C, P> {
    cond: C,
    inner: P,
}

impl<C, P> Cond<C, P>
where
    C: ForwardRefGet<Value = bool>,
{
    pub fn new<Input>(cond: C, inner: P) -> Self
    where
        P: Parser<Input>,
    {
        Self { cond, inner }
    }
}

impl<Input, C, P> Parser<Input> for Cond<C, P>
where
    C: ForwardRefGet<Value = bool>,
    P: Parser<Input>,
{
    type Output = Option<P::Output>;

    fn name(&self) -> String {
        "cond".to_owned()
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
        let mut offset = 0;
        let mut value = None;

        if *self.cond.get() {
            let out;
            (out, offset, child_annotations) = self.inner.parse_with(input, annotation_mode).fold(
                annotation_mode,
                || self.name(),
                child_annotations,
                offset,
                0,
            )?;

            value = Some(out);
        }

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
