use crate::Annotation;
use crate::AnnotationResult;
use crate::ParserSpec;
use crate::combinators::delayed::DelayedParser;
use std::fmt::Debug;
use std::ops::Range;

pub type AnnotatedResult<T> = std::result::Result<(T, Annotation), Annotation>;

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

pub type ParseResult<T> = std::result::Result<(T, usize), Annotation>;

/// All parsing functions must implement this trait
pub trait Parser<Input> {
    /// Debug/Clone as we store a copy in the return annotations
    type Output: Debug + Clone + 'static;

    /// Simple name of the parser, should not include children or generics
    // TODO: Change this to a CoW so we're not constantly copying `&'static str`s
    fn name(&self) -> String;

    /// A static representation of the parser structure
    fn spec(&self) -> ParserSpec;

    /// Configure which paths get annotated
    fn parse_with(
        &mut self,
        _input: &mut Input,
        _annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        todo!()
    }

    /// Parse and return both the output value and annotations
    #[inline(always)]
    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        match self.parse_with(input, AnnotationMode::ALL) {
            Ok((value, anno)) => Ok((value, anno.annotation())),
            Err(anno) => Err(anno.annotation()),
        }
    }

    /// "Fast" implementation of the parser, only producing annotations on error
    /// Default impl just runs the slow version and strips off annotations.  
    ///
    /// Failure case only needs to return annotations for the failure branch.  
    /// Eg. for a LengthRepeat(u32, u16), if the 5th application of the u16 parser fails, the
    /// returned annotation should look roughly like:  
    /// ```ignore
    ///     Anno::Child {
    ///         name: "length_repeat",
    ///         start: 0,
    ///         children: [
    ///             Anno::Incomplete {
    ///                 name: "u16",
    ///                 start: 12,
    ///             }
    ///         ]
    ///     }
    /// ````
    #[inline(always)]
    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        match self.parse_with(input, AnnotationMode::FAIL) {
            Ok((value, anno)) => Ok((value, anno.span().end)),
            Err(anno) => Err(anno.annotation()),
        }
    }
}

pub type ParseWithResult<T> = std::result::Result<(T, AnnotationReturn), AnnotationReturn>;

#[derive(Debug)]
pub enum AnnotationReturn {
    // TODO: Box<Annotation> so stack size isn't massive? 112 vs 24 bytes
    Annotated(Annotation),
    Span(Range<usize>),
    Start(usize),
}

impl AnnotationReturn {
    pub fn annotation(self) -> Annotation {
        let Self::Annotated(a) = self else {
            unreachable!()
        };
        a
    }

    pub fn span(self) -> Range<usize> {
        let Self::Span(span) = self else {
            unreachable!()
        };
        span
    }

    pub fn start(self) -> usize {
        let Self::Start(start) = self else {
            unreachable!()
        };
        start
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AnnotationMode {
    pub success: bool,
    pub fail: bool,
}

impl AnnotationMode {
    pub const NONE: Self = AnnotationMode {
        success: false,
        fail: false,
    };
    pub const ALL: Self = AnnotationMode {
        success: true,
        fail: true,
    };
    pub const FAIL: Self = AnnotationMode {
        success: false,
        fail: true,
    };
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        (**self).annotate(input)
    }

    fn parse(&mut self, input: &mut Input) -> ParseResult<Self::Output> {
        (**self).parse(input)
    }
}

impl<Input, P> DelayedParser<Input> for Box<P>
where
    P: DelayedParser<Input>,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        (**self).annotate(input)
    }

    fn parse(&mut self, input: &mut Input) -> ParseResult<Self::Output> {
        (**self).parse(input)
    }
}
