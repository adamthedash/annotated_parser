use std::fmt::{Debug, Display};

use crate::{AnnotatedResult, Annotation, FoldResult, Parser, ParserSpec, helpers::fold_child_err};

/// For fallible functions
pub struct TryMap<I, F> {
    inner: I,
    func: F,
}

impl<I, F> TryMap<I, F> {
    pub fn new(inner: I, func: F) -> Self {
        Self { inner, func }
    }
}

impl<Input, I, F, O, E> Parser<Input> for TryMap<I, F>
where
    I: Parser<Input>,
    F: FnMut(I::Output) -> std::result::Result<O, E>,
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
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

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        let (data, offset, child_annotations) =
            self.inner
                .annotate(input)
                .fold(vec![], 0, || self.name(), 0)?;

        let value = (self.func)(data);

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
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

impl<I, F> MapSilent<I, F> {
    pub fn new(inner: I, func: F) -> Self {
        Self { inner, func }
    }
}

impl<Input, I, F, O> Parser<Input> for MapSilent<I, F>
where
    I: Parser<Input>,
    F: FnMut(I::Output) -> O,
    O: Debug + Clone + 'static,
{
    type Output = O;

    fn name(&self) -> String {
        self.inner.name()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec()
    }

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        let (data, annotation) = self.inner.annotate(input)?;

        let out = (self.func)(data);

        Ok((out, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let (data, offset) = self.inner.parse(input)?;

        let out = (self.func)(data);

        Ok((out, offset))
    }
}
