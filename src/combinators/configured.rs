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
/// Use `configure_with` to create a closure that updates the flag from a `ForwardRef`.
/// If disabled, returns `None` without consuming input.
///
/// This is designed to be used with [`Configuring`]: a value is parsed once,
/// the parser is configured based on that value, and then the configured parser
/// is applied repeatedly. Unlike [`Cond`], the condition is checked once after
/// the value is read, making it efficient for repeated application.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::combinators::{Configured, Configuring};
/// use annotated_parser::combinators::Store;
/// use annotated_parser::ByteParser;
///
/// let version = u8::LE.store();
///
/// // Only present when version >= 1
/// let extra = Configured::new(u8::LE);
/// let configure_extra = extra.configure_with(
///     version.output().map(|v| *v >= 1)
/// );
///
/// // Configure extra after the version is read
/// let version = Configuring::new(version, || {
///     configure_extra();
/// });
///
/// let mut parser = (version, extra);
/// let mut input = &[1, 42][..];
/// let ((ver, ext), _) = parser.parse(&mut input).unwrap();
/// assert_eq!(ver, 1);
/// assert_eq!(ext, Some(42));
/// ```
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
/// The configurator is run after the inner parser succeeds.
/// This is typically used with [`Configured`] to set up flags after a value is parsed.
/// The key difference from [`Cond`] is that the condition is evaluated once after parsing,
/// making it efficient for repeated application of the configured parser.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::combinators::{Configured, Configuring};
/// use annotated_parser::combinators::Store;
/// use annotated_parser::ByteParser;
///
/// let version = u8::LE.store();
///
/// // Only present when version >= 1
/// let extra = Configured::new(u8::LE);
/// let configure_extra = extra.configure_with(
///     version.output().map(|v| *v >= 1)
/// );
///
/// // Configure extra after the version is read
/// let version = Configuring::new(version, || {
///     configure_extra();
/// });
///
/// let mut parser = (version, extra);
/// let mut input = &[1, 42][..];
/// let ((ver, ext), _) = parser.parse(&mut input).unwrap();
/// assert_eq!(ver, 1);
/// assert_eq!(ext, Some(42));
/// ```
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
