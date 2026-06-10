use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec, StoringParser,
    combinators::store::ForwardRefGet, helpers::FoldParseWithResult, parser::ParseWithResult,
};

/// A parser that can be externally enabled or disabled.
///
/// The parser is only executed when the internal `AtomicBool` is `true`.
/// Use `configure_with` to create a closure that updates the flag from a `ForwardRef`.
/// If disabled, returns `None` without consuming input.
///
/// This is useful for feature-gated or runtime-configurable parser sections.
pub struct Configured<P> {
    enabled: Arc<AtomicBool>,
    inner: P,
}

impl<P> Configured<P> {
    pub fn new<Input>(inner: P) -> Self
    where
        P: Parser<Input>,
    {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            inner,
        }
    }

    /// Create an on-demand configuring closure. Calling the returned closure will update the enabled
    /// status of this combinator based off the current value of the provided reference value
    pub fn configure_with<T>(&self, val: T) -> impl Fn() + use<P, T>
    where
        T: ForwardRefGet<Value = bool>,
    {
        let enabled = self.enabled.clone();
        move || {
            enabled.store(*val.get(), Ordering::Relaxed);
        }
    }
}

impl<Input, P> Parser<Input> for Configured<P>
where
    P: Parser<Input>,
{
    type Output = Option<P::Output>;

    fn name(&self) -> String {
        "configured".to_owned()
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

        if self.enabled.load(Ordering::Relaxed) {
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

/// Run a parser, then execute a side-effect closure.
///
/// Applies the inner parser normally, then runs the configurator function.
/// This is useful for one-off configuration of future parsers after a value has been parsed.
///
/// The configurator is run after parsing, so it can inspect or set up state based on the result.
pub struct Configuring<P, F> {
    inner: P,
    configurator: F,
}

impl<P, F> Configuring<P, F> {
    pub fn new<Input>(inner: P, configurator: F) -> Self
    where
        P: Parser<Input>,
        F: Fn(),
    {
        Self {
            inner,
            configurator,
        }
    }
}

impl<Input, P, F> Parser<Input> for Configuring<P, F>
where
    P: Parser<Input>,
    F: Fn(),
{
    type Output = P::Output;

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
        let res = self.inner.parse_with(input, annotation_mode)?;
        (self.configurator)();
        Ok(res)
    }
}

impl<Input, P, F> StoringParser<Input> for Configuring<P, F>
where
    P: StoringParser<Input>,
    F: Fn(),
{
    type Value = P::Value;

    type Ref = P::Ref;

    fn output(&self) -> Self::Ref {
        self.inner.output()
    }
}
