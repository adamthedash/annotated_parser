use std::fmt::{Debug, Display};

use crate::{AnnotatedResult, Annotation, FoldResult, Parser, ParserSpec, helpers::fold_child_err};

/// For fallible functions
pub struct TryMap<I, F> {
    inner: I,
    func: F,
}

pub(crate) fn try_map<'a, I, F, O, E>(inner: I, func: F) -> TryMap<I, F>
where
    I: Parser<'a>,
    F: FnMut(I::Output) -> std::result::Result<O, E>,
    O: Debug + Clone + 'static,
    E: Display,
{
    // NOTE: Free function so we don't need to pass more types to the TryMap struct
    TryMap { inner, func }
}

impl<'a, I, F, O, E> Parser<'a> for TryMap<I, F>
where
    I: Parser<'a>,
    F: FnMut(I::Output) -> std::result::Result<O, E>,
    O: Debug + Clone + 'static,
    E: Display,
{
    type Input = I::Input;

    type Output = O;

    fn name(&self) -> String {
        "try_map".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
        let (data, offset, child_annotations) =
            self.inner
                .annotate(input)
                .fold(vec![], 0, || self.name(), 0)?;

        let out = match (self.func)(data) {
            Ok(value) => value,
            Err(e) => {
                // Function application has failed, so fail annotation at this level
                return Err(Annotation::invalid(
                    self.name(),
                    0..offset,
                    format!("{}", e),
                    child_annotations,
                ));
            }
        };

        let annotation =
            Annotation::success(self.name(), 0..offset, out.clone(), child_annotations);

        Ok((out, annotation))
    }

    fn parse(&mut self, input: &mut Self::Input) -> crate::ParseResult<Self::Output> {
        let (data, offset) = self
            .inner
            .parse(input)
            .map_err(|a| fold_child_err(a, vec![], 0, self.name(), 0))?;

        let out = (self.func)(data)
            // Function application has failed, so fail annotation at this level
            .map_err(|e| Annotation::invalid(self.name(), 0..offset, format!("{}", e), vec![]))?;

        Ok((out, offset))
    }
}

/// For infallible functions
pub struct Map<I, F> {
    inner: I,
    func: F,
}

pub(crate) fn map<'a, I, F, O>(inner: I, func: F) -> Map<I, F>
where
    I: Parser<'a>,
    F: FnMut(I::Output) -> O,
    O: Debug + Clone + 'static,
{
    // NOTE: Free function so we don't need to pass more types to the Map struct
    Map { inner, func }
}

impl<'a, I, F, O> Parser<'a> for Map<I, F>
where
    I: Parser<'a>,
    F: FnMut(I::Output) -> O,
    O: Debug + Clone + 'static,
{
    type Input = I::Input;

    type Output = O;

    fn name(&self) -> String {
        "map".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
        let (data, offset, child_annotations) =
            self.inner
                .annotate(input)
                .fold(vec![], 0, || self.name(), 0)?;

        let value = (self.func)(data);

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut Self::Input) -> crate::ParseResult<Self::Output> {
        let (data, offset) = self
            .inner
            .parse(input)
            .map_err(|a| fold_child_err(a, vec![], 0, self.name(), 0))?;

        let out = (self.func)(data);

        Ok((out, offset))
    }
}

/// For infallible functions. Doesn't introduce anything new in the spec. Can be used for simple
/// functions where it would just add noise to track them in the annotations
pub struct MapSilent<I, F> {
    inner: I,
    func: F,
}

pub(crate) fn map_silent<'a, I, F, O>(inner: I, func: F) -> MapSilent<I, F>
where
    I: Parser<'a>,
    F: FnMut(I::Output) -> O,
    O: Debug + Clone + 'static,
{
    // NOTE: Free function so we don't need to pass more types to the MapSilent struct
    MapSilent { inner, func }
}

impl<'a, I, F, O> Parser<'a> for MapSilent<I, F>
where
    I: Parser<'a>,
    F: FnMut(I::Output) -> O,
    O: Debug + Clone + 'static,
{
    type Input = I::Input;

    type Output = O;

    fn name(&self) -> String {
        self.inner.name()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec()
    }

    fn annotate(&mut self, input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
        let (data, annotation) = self.inner.annotate(input)?;

        let out = (self.func)(data);

        Ok((out, annotation))
    }

    fn parse(&mut self, input: &mut Self::Input) -> crate::ParseResult<Self::Output> {
        let (data, offset) = self.inner.parse(input)?;

        let out = (self.func)(data);

        Ok((out, offset))
    }
}
