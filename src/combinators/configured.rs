use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{
    Annotation, FoldResult, Parser, ParserSpec, combinators::delayed::DelayedValGet,
    helpers::fold_child_err,
};

/// Parser which can be externally enabled/disabled rather than checking a delayed value each
/// execution
pub struct Configured<P>
where
    P: Parser,
{
    enabled: Arc<AtomicBool>,
    inner: P,
}

impl<P> Configured<P>
where
    P: Parser,
{
    pub fn new(inner: P) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            inner,
        }
    }

    /// Create an on-demand configure closure. Calling the returned closure will update the enabled
    /// status of this combinator based off the current value of the provided delayed value
    pub fn configure_with<T>(&self, val: T) -> impl Fn() + use<P, T>
    where
        T: DelayedValGet<Value = bool>,
    {
        let enabled = self.enabled.clone();
        move || {
            enabled.store(*val.get(), Ordering::Relaxed);
        }
    }
}

impl<P> Parser for Configured<P>
where
    P: Parser,
{
    type Output = Option<P::Output>;

    fn name(&self) -> String {
        "configured".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut &[u8]) -> crate::AnnotatedResult<Self::Output> {
        let (value, offset, child_annotations) = if self.enabled.load(Ordering::Relaxed) {
            let (value, offset, child_annotations) =
                self.inner
                    .annotate(input)
                    .fold(vec![], 0, || self.name(), 0)?;

            (Some(value), offset, child_annotations)
        } else {
            (None, 0, vec![])
        };

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);
        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Ok((None, 0));
        }

        let (value, offset) = self
            .inner
            .parse(input)
            .map_err(|a| fold_child_err(a, vec![], 0, self.name(), 0))?;

        Ok((Some(value), offset))
    }
}

/// A combinator which first runs the inner parser, then runs the configuring function
/// This can be used to perform a one-off configuring of future parsers
pub struct Configuring<P, F> {
    inner: P,
    configurator: F,
}

impl<P, F> Configuring<P, F>
where
    P: Parser,
    F: Fn(),
{
    pub fn new(inner: P, configurator: F) -> Self {
        Self {
            inner,
            configurator,
        }
    }
}

impl<P, F> Parser for Configuring<P, F>
where
    P: Parser,
    F: Fn(),
{
    type Output = P::Output;

    fn name(&self) -> String {
        self.inner.name()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec()
    }

    fn annotate(&mut self, input: &mut &[u8]) -> crate::AnnotatedResult<Self::Output> {
        let res = self.inner.annotate(input)?;
        (self.configurator)();
        Ok(res)
    }

    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let res = self.inner.parse(input)?;
        (self.configurator)();
        Ok(res)
    }
}
