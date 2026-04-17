use crate::{
    Annotation, AnnotationMode, AnnotationResult, AnnotationReturn, Parser, ParserSpec,
    combinators::delayed::DelayedParser, parser::ParseWithResult,
};

/// For adding a user-friendly name to the spec
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

    #[inline(always)]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        self.inner.parse_with(input, annotation_mode)
    }
}

impl<Input, P> DelayedParser<Input> for Trace<P>
where
    P: DelayedParser<Input>,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
        self.inner.output()
    }
}

/// Overrides the inner parser with a friendly name. Does not propagate spec or annotations from
/// inner parser upwards. From the user's perspective, this becomes a "base" parser. This can be
/// useful to reduce noise in the output for complex combinators where the inner workings aren't
/// that relevant. Eg. a whitespace parser consisting of Take + Verify + Repeat
// TODO: Impl - also maybe rename to BlackBox?
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

    #[inline(always)]
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
                    Annotation::success(self.name(), annotation.span(), value.clone(), vec![])
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
                    let mut annotation = annotation.annotation();
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
