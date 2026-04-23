use std::fmt::{Debug, Display};

use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec, helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

/// For fallible functions
pub struct TryMap<P, F> {
    inner: P,
    func: F,
}

impl<P, F> TryMap<P, F> {
    pub fn new<Input, O, E>(inner: P, func: F) -> Self
    where
        P: Parser<Input>,
        F: FnMut(P::Output) -> std::result::Result<O, E>,
        O: Debug + Clone + 'static,
        E: Display,
    {
        Self { inner, func }
    }
}

impl<Input, P, F, O, E> Parser<Input> for TryMap<P, F>
where
    P: Parser<Input>,
    F: FnMut(P::Output) -> std::result::Result<O, E>,
    O: Debug + Clone + 'static,
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

/// For infallible functions
pub struct Map<P, F> {
    inner: P,
    func: F,
}

impl<P, F> Map<P, F> {
    pub fn new<Input, O>(inner: P, func: F) -> Self
    where
        P: Parser<Input>,
        F: FnMut(P::Output) -> O,
        O: Debug + Clone + 'static,
    {
        Self { inner, func }
    }
}

impl<Input, P, F, O> Parser<Input> for Map<P, F>
where
    P: Parser<Input>,
    F: FnMut(P::Output) -> O,
    O: Debug + Clone + 'static,
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

/// For infallible functions. Doesn't introduce anything new in the spec. Can be used for simple
/// functions where it would just add noise to track them in the annotations
pub struct MapSilent<P, F> {
    inner: P,
    func: F,
}

impl<P, F> MapSilent<P, F> {
    pub fn new<Input, O>(inner: P, func: F) -> Self
    where
        P: Parser<Input>,
        F: FnMut(P::Output) -> O,
        O: Debug + Clone + 'static,
    {
        Self { inner, func }
    }
}

impl<Input, P, F, O> Parser<Input> for MapSilent<P, F>
where
    P: Parser<Input>,
    F: FnMut(P::Output) -> O,
    O: Debug + Clone + 'static,
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
