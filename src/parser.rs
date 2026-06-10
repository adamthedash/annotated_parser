use crate::Annotation;
use crate::ParserSpec;
use crate::StoringParser;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Range;

/// Result type used by [`Parser::annotate`].
///
/// Returns the parsed value and a full [`Annotation`] tree on success,
/// or the failure [`Annotation`] on error.
pub type AnnotatedResult<T> = std::result::Result<(T, Annotation), Annotation>;

/// Consume an annotated result and return just the [`Annotation`].
pub trait IntoAnnotation {
    /// Consume the result and return the annotation from either branch
    fn into_annotation(self) -> Annotation;
}

impl<T> IntoAnnotation for AnnotatedResult<T> {
    fn into_annotation(self) -> Annotation {
        match self {
            Ok((_, a)) => a,
            Err(a) => a,
        }
    }
}

/// Result type used by [`Parser::parse`].
///
/// Returns the parsed value and the consumed byte count on success,
/// or a failure [`Annotation`] on error.
pub type ParseResult<T> = std::result::Result<(T, usize), Annotation>;

/// Result type used by [`Parser::parse_with`].
///
/// Returns the parsed value on success, along with an annotation of varying verbosity depending on
/// the AnnotationMode used.
pub type ParseWithResult<T> = std::result::Result<(T, AnnotationReturn), AnnotationReturn>;

/// A type that can be safely cloned and boxed up for annotations.
///
/// `Debug` is needed for display, `Clone` because the value is stored in
/// success annotations, and `Send + Sync + 'static` because it may be boxed
/// as a trait object and shared across threads.
pub trait ParserOutput: Debug + Clone + Send + Sync + 'static {}
impl<T> ParserOutput for T where T: Debug + Clone + Send + Sync + 'static {}

/// The core trait that all parsers must implement.
///
/// A `Parser` defines how to consume an input and produce a value. It also
/// provides metadata about its structure.
pub trait Parser<Input> {
    /// The type produced on success.
    type Output: ParserOutput;

    /// Simple name of the parser, used as the base identifier in the annotation tree.
    ///
    /// Should not include children or generics; those are handled by [`ParserSpec`].
    // TODO: Change this to a CoW so we're not constantly copying `&'static str`s
    fn name(&self) -> String;

    /// A static representation of the parser structure.
    ///
    /// Mirrors the parser hierarchy (leaf parsers, combinators, nested
    /// combinators) without holding any runtime state.
    fn spec(&self) -> ParserSpec;

    /// Low-level parse method with full annotation control.
    ///
    /// This is the single method that all other entry points (`parse`, `annotate`)
    /// build on top of. It is intended for custom parser and combinator authors
    /// who need fine-grained control over which paths are annotated.
    ///
    /// The `annotation_mode` parameter controls whether success and/or failure
    /// paths are annotated. See [`AnnotationMode`] for the available modes.
    fn parse_with(
        &mut self,
        _input: &mut Input,
        _annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output>;

    /// Parse and return both the output value and the full annotation tree.
    ///
    /// This is the slow path: it collects annotations for every parser in the
    /// hierarchy, regardless of success or failure. Useful for debugging and
    /// visualising the parse trace.
    #[inline]
    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        match self.parse_with(input, AnnotationMode::ALL) {
            Ok((value, anno)) => Ok((value, anno.annotation().expect("Annotated path"))),
            Err(anno) => Err(anno.annotation().expect("Annotated path")),
        }
    }

    /// "Fast" parse: only produces annotations on failure. The returned annotation
    /// contains only the hierarchy leading to the failure source.
    ///
    /// For example, if `LengthRepeat(u32, u16)` fails because the 5th `u16`
    /// parse fails, the returned annotation should look roughly like:
    /// ```ignore
    ///     Annotation::Child {
    ///         name: "length_repeat",
    ///         start: 0,
    ///         children: [
    ///             Annotation::Incomplete {
    ///                 name: "u16",
    ///                 start: 12,
    ///             }
    ///         ]
    ///     }
    /// ```
    #[inline]
    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        match self.parse_with(input, AnnotationMode::FAIL) {
            Ok((value, anno)) => Ok((value, anno.span().expect("Unannoated path").end)),
            Err(anno) => Err(anno.annotation().expect("Annotated path")),
        }
    }
}

/// Returned annotation type for [`parse_with`](Parser::parse_with).
///
/// Contains varying levels of annotation information depending on what AnnotationMode was used.
#[derive(Debug)]
pub enum AnnotationReturn {
    /// Fully annotated success or failure.
    Annotated(Annotation),
    /// Unannotated success, or an invalid failure with a span.
    Span(Range<usize>),
    /// Unannotated incomplete failure with a starting offset.
    Start(usize),
}

impl AnnotationReturn {
    /// Extract the full annotation, if this is an annotated result.
    pub fn annotation(self) -> Option<Annotation> {
        if let Self::Annotated(a) = self {
            Some(a)
        } else {
            None
        }
    }

    /// Extract the span, if this is a `Span` result.
    pub fn span(self) -> Option<Range<usize>> {
        if let Self::Span(span) = self {
            Some(span)
        } else {
            None
        }
    }

    /// Extract the starting offset from `Span` or `Start` results.
    pub fn start(self) -> Option<usize> {
        let start = match self {
            Self::Span(span) => span.start,
            Self::Start(start) => start,
            _ => return None,
        };
        Some(start)
    }
}

impl From<Annotation> for AnnotationReturn {
    fn from(value: Annotation) -> Self {
        Self::Annotated(value)
    }
}

impl std::error::Error for AnnotationReturn {}

impl Display for AnnotationReturn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnnotationReturn::Annotated(annotation) => write!(f, "Parse failure: {:?}", annotation),
            AnnotationReturn::Span(range) => write!(f, "Parse failure at {:?}", range),
            AnnotationReturn::Start(start) => write!(f, "Parse failure starting at {start}"),
        }
    }
}

/// Controls which paths are annotated during parsing
#[derive(Debug, Clone, Copy)]
pub struct AnnotationMode {
    /// Whether success paths are annotated.
    pub success: bool,
    /// Whether failure paths are annotated.
    pub fail: bool,
}

impl AnnotationMode {
    /// No annotations at all.
    pub const NONE: Self = AnnotationMode {
        success: false,
        fail: false,
    };
    /// Annotate both success and failure paths.
    pub const ALL: Self = AnnotationMode {
        success: true,
        fail: true,
    };
    /// Annotate only failure paths.
    pub const FAIL: Self = AnnotationMode {
        success: false,
        fail: true,
    };
    /// Annotate only success paths.
    pub const SUCCESS: Self = AnnotationMode {
        success: true,
        fail: false,
    };
}

/// Blanket impl for boxed parsers
impl<Input, P> Parser<Input> for Box<P>
where
    P: Parser<Input> + ?Sized,
{
    type Output = P::Output;

    fn name(&self) -> String {
        (**self).name()
    }

    fn spec(&self) -> ParserSpec {
        (**self).spec()
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        (**self).parse_with(input, annotation_mode)
    }
}

impl<Input, P> StoringParser<Input> for Box<P>
where
    P: StoringParser<Input>,
{
    type Value = P::Value;
    type Ref = P::Ref;

    fn output(&self) -> Self::Ref {
        (**self).output()
    }
}

/// Blanket impl to allow passing parsers by reference
impl<Input, P> Parser<Input> for &mut P
where
    P: Parser<Input>,
{
    type Output = P::Output;

    fn name(&self) -> String {
        (**self).name()
    }

    fn spec(&self) -> ParserSpec {
        (**self).spec()
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        (**self).parse_with(input, annotation_mode)
    }
}
