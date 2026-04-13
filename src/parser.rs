use crate::Annotation;
use crate::AnnotationResult;
use crate::ParserSpec;
use crate::combinators::delayed::DelayedParser;
use std::fmt::Debug;

pub type Result<T> = std::result::Result<(T, Annotation), Annotation>;
pub type SpeedyResult<T> = std::result::Result<(T, usize), Annotation>;

/// All parsing functions must implement this trait
pub trait Parser {
    type Output: Debug + Clone + 'static;

    /// Simple name of the parser, should not include children or generics
    // TODO: Change this to a CoW so we're not constantly copying `&'static str`s
    fn name(&self) -> String;

    /// A static representation of the parser structure
    fn spec(&self) -> ParserSpec;

    /// Parse and return both the output value and annotations
    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output>;

    /// Parse and just return the annotations
    fn annotate(&mut self, mut input: &[u8]) -> Annotation {
        match self.parse(&mut input) {
            Ok((_, a)) => a,
            Err(a) => a,
        }
    }

    /// "Fast" implementation of the parser, only producing annotations on error
    /// Default impl just runs the slow version and strips off annotations.  
    ///
    /// Failure case only needs to return annotations for the failure branch.  
    /// Eg. for a LengthRepeat(u32, u16), if the 5th application of the u16 parser fails, the
    /// returned annotation should look roughly like:  
    /// ```
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
    fn parse_speedy(&mut self, input: &mut &[u8]) -> SpeedyResult<Self::Output> {
        match self.parse(input) {
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
impl<P> Parser for Box<P>
where
    P: Parser + ?Sized,
{
    type Output = P::Output;

    fn name(&self) -> String {
        (**self).name()
    }

    fn spec(&self) -> ParserSpec {
        (**self).spec()
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        (**self).parse(input)
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> SpeedyResult<Self::Output> {
        (**self).parse_speedy(input)
    }
}

impl<P> DelayedParser for Box<P>
where
    P: DelayedParser,
{
    type Value = P::Value;
    type DelayedValue = P::DelayedValue;

    fn output(&self) -> Self::DelayedValue {
        (**self).output()
    }
}

/// Blanket impl to allow passing parsers by reference
impl<P> Parser for &mut P
where
    P: Parser,
{
    type Output = P::Output;

    fn name(&self) -> String {
        (**self).name()
    }

    fn spec(&self) -> ParserSpec {
        (**self).spec()
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        (**self).parse(input)
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> SpeedyResult<Self::Output> {
        (**self).parse_speedy(input)
    }
}
