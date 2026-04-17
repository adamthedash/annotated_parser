use crate::Annotation;
use crate::AnnotationResult;
use crate::ParserSpec;
use crate::combinators::delayed::DelayedParser;
use std::fmt::Debug;

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

    /// Parse and return both the output value and annotations
    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output>;

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
    fn parse(&mut self, input: &mut Input) -> ParseResult<Self::Output> {
        // For checking when I forget to impl things fully
        // panic!("default parse impl: {:?}", std::any::type_name::<Self>());

        match self.annotate(input) {
            Ok((v, a)) => {
                let AnnotationResult::Success { span, .. } = a.result else {
                    unreachable!("Parser succeeded");
                };

                Ok((v, span.end))
            }
            Err(a) => Err(a),
        }
    }
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
