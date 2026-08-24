use crate::{
    Annotation, AnnotationMode, AnnotationResult, AnnotationReturn, Parser, ParserSpec,
    combinators::store::StoringParser, parser::ParseWithResult,
};

/// Add a user-friendly name to a parser's spec.
///
/// Transparent wrapper: the inner parser's behavior is unchanged.
/// The name appears in the parser spec and annotations, making traces more readable.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::ByteParser;
/// use annotated_parser::combinators::Trace;
///
/// let mut parser = Trace::new(u8::LE, "byte");
/// let mut input = &[1_u8][..];
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, 1);
/// ```
#[derive(Clone)]
pub struct Trace<P> {
    inner: P,
    name: String,
}

impl<P> Trace<P> {
    pub fn new<Input>(inner: P, name: impl Into<String>) -> Self
    where
        P: Parser<Input>,
    {
        Self {
            inner,
            name: name.into(),
        }
    }
}

impl<Input, P> Parser<Input> for Trace<P>
where
    P: Parser<Input>,
{
    type Output = P::Output;

    fn name(&self) -> String {
        // TODO: Pass through inner name?
        //  Or "trace"?
        //  Or self.name?
        self.name.clone()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec().with_friendly(self.name())
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        self.inner.parse_with(input, annotation_mode)
    }
}

impl<Input, P> StoringParser<Input> for Trace<P>
where
    P: StoringParser<Input>,
{
    type Value = P::Value;
    type Ref = P::Ref;

    fn output(&self) -> Self::Ref {
        self.inner.output()
    }
}

/// Hide a parser's internal details from the annotation tree.
///
/// The inner parser is treated as an opaque base parser.
/// Success annotations show the final span and value, but the internal hierarchy is hidden.
/// Failure annotations propagate with a simplified error message.
///
/// This is useful for reducing noise from complex sub-parsers like whitespace or checksum logic.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::ByteParser;
/// use annotated_parser::combinators::TraceOpaque;
///
/// let mut parser = TraceOpaque::new(u8::LE, "byte");
/// let mut input = &[1_u8][..];
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, 1);
/// ```
pub struct TraceOpaque<P> {
    inner: P,
    name: String,
}

impl<P> TraceOpaque<P> {
    pub fn new<Input>(inner: P, name: impl Into<String>) -> Self
    where
        P: Parser<Input>,
    {
        Self {
            inner,
            name: name.into(),
        }
    }
}

impl<Input, P> Parser<Input> for TraceOpaque<P>
where
    P: Parser<Input>,
{
    type Output = P::Output;

    fn name(&self) -> String {
        "trace_opaque".to_string()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name()).with_friendly(self.name.clone())
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let inner_mode = AnnotationMode {
            success: false,
            fail: annotation_mode.fail,
        };

        match self.inner.parse_with(input, inner_mode) {
            Ok((value, annotation)) => {
                let annotation = if annotation_mode.success {
                    Annotation::success(
                        self.name(),
                        annotation.span().expect("Inner un-annotated path"),
                        value.clone(),
                        vec![],
                    )
                    .into()
                } else {
                    annotation
                };

                Ok((value, annotation))
            }
            Err(annotation) => {
                let annotation = if annotation_mode.fail {
                    // Materialise the failure case to give some indication to user where the
                    // internal failure happened
                    let mut annotation = annotation.annotation().expect("Annotated path");
                    annotation.materialize();

                    let Some(source) = annotation.err_source() else {
                        unreachable!("Failure path");
                    };

                    match &source.result {
                        AnnotationResult::Incomplete { .. } => {
                            Annotation::incomplete(self.name(), 0, vec![])
                        }
                        AnnotationResult::Invalid { span, reason } => Annotation::invalid(
                            self.name(),
                            0..span.end,
                            format!("{} @ {:?}: {} ", source.parser_id, span, reason),
                            vec![],
                        ),
                        AnnotationResult::OOM { requested, .. } => {
                            Annotation::oom(self.name(), 0, *requested)
                        }
                        AnnotationResult::Success { .. } | AnnotationResult::Child { .. } => {
                            unreachable!("At failure source")
                        }
                    }
                    .into()
                } else {
                    match annotation {
                        AnnotationReturn::Span(range) => AnnotationReturn::Span(0..range.end),
                        AnnotationReturn::Start(_) => AnnotationReturn::Start(0),
                        AnnotationReturn::Annotated(_) => unreachable!(),
                    }
                };

                Err(annotation)
            }
        }
    }
}

impl<Input, P> StoringParser<Input> for TraceOpaque<P>
where
    P: StoringParser<Input>,
{
    type Value = P::Value;
    type Ref = P::Ref;

    fn output(&self) -> Self::Ref {
        self.inner.output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    use crate::{
        AnnotationResult, ByteParser, IntoAnnotation, Parser, ParserAdapter,
        combinators::LengthRepeat,
    };

    #[test]
    fn test_good() {
        let parser = LengthRepeat::new(u8::LE, u16::LE.verify(|x| *x == 0));
        let mut parser = TraceOpaque::new(parser, "opaque");

        let input = [1, 0, 0];

        let annotation = parser.annotate(&mut input.as_slice()).into_annotation();
        println!("{:#?}", annotation);
        assert!(matches!(
            annotation.result,
            AnnotationResult::Success {
                span: Range { start: 0, end: 3 },
                ..
            }
        ));
    }

    #[test]
    fn test_incomplete() {
        let parser = LengthRepeat::new(u8::LE, u16::LE.verify(|x| *x == 0));
        let mut parser = TraceOpaque::new(parser, "opaque");

        let input = [2, 0, 0];

        let annotation = parser.annotate(&mut input.as_slice()).into_annotation();
        assert!(matches!(
            annotation.result,
            AnnotationResult::Incomplete { start: 0, .. }
        ));
    }

    #[test]
    fn test_invalid() {
        let parser = LengthRepeat::new(u8::LE, u16::LE.verify(|x| *x == 0));
        let mut parser = TraceOpaque::new(parser, "opaque");

        let input = [2, 0, 1];

        let annotation = parser.annotate(&mut input.as_slice()).into_annotation();
        assert!(matches!(
            annotation.result,
            AnnotationResult::Invalid {
                span: Range { start: 0, end: 3 },
                ..
            }
        ));
    }
}
