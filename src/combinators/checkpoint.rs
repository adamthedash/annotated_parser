use crate::{AnnotationMode, Parser, ParserSpec, combinators::store::StoringParser};

/// Wrapper which resets the input stream on failure
pub struct Checkpoint<P>(P);

impl<P> Checkpoint<P> {
    pub fn new<Input>(inner: P) -> Self
    where
        P: Parser<Input>,
        Input: Copy,
    {
        Self(inner)
    }
}

impl<Input, P> Parser<Input> for Checkpoint<P>
where
    P: Parser<Input>,
    Input: Copy,
{
    type Output = P::Output;

    fn name(&self) -> String {
        self.0.name()
    }

    fn spec(&self) -> ParserSpec {
        self.0.spec()
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> Result<(Self::Output, crate::AnnotationReturn), crate::AnnotationReturn> {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        let res = self.0.parse_with(input, annotation_mode);
        if res.is_err() {
            // Reset input
            *input = checkpoint;
        }

        res
    }
}

impl<Input, P> StoringParser<Input> for Checkpoint<P>
where
    P: StoringParser<Input>,
    Input: Copy,
{
    type Value = P::Value;
    type Ref = P::Ref;

    fn output(&self) -> Self::Ref {
        self.0.output()
    }
}

/// Wrapper which resets the input stream in all cases
pub struct Peek<P>(P);

impl<P> Peek<P> {
    pub fn new<Input>(inner: P) -> Self
    where
        P: Parser<Input>,
        Input: Copy,
    {
        Self(inner)
    }
}

impl<Input, P> Parser<Input> for Peek<P>
where
    P: Parser<Input>,
    Input: Copy,
{
    type Output = P::Output;

    fn name(&self) -> String {
        self.0.name()
    }

    fn spec(&self) -> ParserSpec {
        self.0.spec()
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> Result<(Self::Output, crate::AnnotationReturn), crate::AnnotationReturn> {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        // TODO: On success this will return an annotation in the "future", so it might conflict
        // with follow-on annotations. Maybe return 0-span annotation instead?
        let res = self.0.parse_with(input, annotation_mode);

        // Reset input
        *input = checkpoint;

        res
    }
}

impl<Input, P> StoringParser<Input> for Peek<P>
where
    P: StoringParser<Input>,
    Input: Copy,
{
    type Value = P::Value;
    type Ref = P::Ref;

    fn output(&self) -> Self::Ref {
        self.0.output()
    }
}
