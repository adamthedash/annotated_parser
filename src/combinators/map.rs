use std::fmt::Display;

use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserOutput, ParserSpec,
    helpers::FoldParseWithResult, parser::ParseWithResult,
};

/// Apply a fallible function to the output of a parser.
///
/// Runs the inner parser, then applies a function that may fail.
/// If the function returns `Err`, the parser fails with a validation error.
/// For infallible transformations, use [`Map`] instead.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::ByteParser;
///
/// let mut parser = u8::LE.try_map(|x| {
///     if x == 1 { Ok(x) } else { Err("expected 1") }
/// });
/// let mut input = &[1_u8][..];
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, 1);
/// ```
pub struct TryMap<P, F> {
    inner: P,
    func: F,
}

impl<P, F> TryMap<P, F> {
    pub fn new<Input, O, E>(inner: P, func: F) -> Self
    where
        P: Parser<Input>,
        F: FnMut(P::Output) -> std::result::Result<O, E>,
        O: ParserOutput,
        E: Display,
    {
        Self { inner, func }
    }
}

impl<Input, P, F, O, E> Parser<Input> for TryMap<P, F>
where
    P: Parser<Input>,
    F: FnMut(P::Output) -> std::result::Result<O, E>,
    O: ParserOutput,
    E: Display,
{
    type Output = O;

    fn name(&self) -> String {
        "try_map".to_owned()
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
        let (data, offset, child_annotations) =
            self.inner.parse_with(input, annotation_mode).fold(
                annotation_mode,
                || self.name(),
                annotation_mode.success.then(Vec::new),
                0,
                0,
            )?;

        let value = match (self.func)(data) {
            Ok(value) => value,
            Err(e) => {
                let annotation = {
                    if annotation_mode.fail {
                        Annotation::invalid(
                            self.name(),
                            0..offset,
                            format!("{}", e),
                            child_annotations.unwrap_or_default(),
                        )
                        .into()
                    } else {
                        AnnotationReturn::Span(0..offset)
                    }
                };
                return Err(annotation);
            }
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

/// Apply an infallible function to the output of a parser.
///
/// Runs the inner parser, then transforms the result with a user-provided function.
/// The transformed value is included in annotations. For a silent variant that does not
/// add noise to the trace, use [`MapSilent`].
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::ByteParser;
///
/// let mut parser = u8::LE.map(|x| x * 2);
/// let mut input = &[1_u8][..];
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, 2);
/// ```
pub struct Map<P, F> {
    inner: P,
    func: F,
}

impl<P, F> Map<P, F> {
    pub fn new<Input, O>(inner: P, func: F) -> Self
    where
        P: Parser<Input>,
        F: FnMut(P::Output) -> O,
        O: ParserOutput,
    {
        Self { inner, func }
    }
}

impl<Input, P, F, O> Parser<Input> for Map<P, F>
where
    P: Parser<Input>,
    F: FnMut(P::Output) -> O,
    O: ParserOutput,
{
    type Output = O;

    fn name(&self) -> String {
        "map".to_owned()
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
        let (data, offset, child_annotations) =
            self.inner.parse_with(input, annotation_mode).fold(
                annotation_mode,
                || self.name(),
                annotation_mode.success.then(Vec::new),
                0,
                0,
            )?;

        let value = (self.func)(data);

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

/// Apply a silent transformation to the output of a parser.
///
/// Like [`Map`], but does not add any new node to the annotation or spec tree.
/// Useful for lightweight conversions that would just add noise to the trace.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::ByteParser;
///
/// let mut parser = u8::LE.map_silent(|x| x * 2);
/// let mut input = &[1_u8][..];
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, 2);
/// ```
pub struct MapSilent<P, F> {
    inner: P,
    func: F,
}

impl<P, F> MapSilent<P, F> {
    pub fn new<Input, O>(inner: P, func: F) -> Self
    where
        P: Parser<Input>,
        F: FnMut(P::Output) -> O,
        O: ParserOutput,
    {
        Self { inner, func }
    }
}

impl<Input, P, F, O> Parser<Input> for MapSilent<P, F>
where
    P: Parser<Input>,
    F: FnMut(P::Output) -> O,
    O: ParserOutput,
{
    type Output = O;

    fn name(&self) -> String {
        self.inner.name()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec()
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let (data, annotation) = self.inner.parse_with(input, annotation_mode)?;

        let out = (self.func)(data);

        Ok((out, annotation))
    }
}
