use crate::{
    AnnotatedResult, Annotation, AnnotationResult, Parser, ParserSpec,
    combinators::delayed::DelayedParser,
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        self.inner.annotate(input)
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        self.inner.parse(input)
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        match self.inner.parse(input) {
            Ok((value, offset)) => {
                let annotation = Annotation::success(self.name(), 0..offset, value.clone(), vec![]);
                Ok((value, annotation))
            }

            Err(annotation) => {
                // Trim & materialise the failure case to give some indication to user where the
                // internal failure happened
                let mut annotation = annotation.to_failure_tree().expect("Failure path");
                annotation.materialize();

                let Some(source) = annotation.err_source() else {
                    unreachable!("Failure path");
                };
                let annotation = match &source.result {
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
