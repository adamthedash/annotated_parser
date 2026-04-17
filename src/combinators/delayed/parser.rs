use super::value::DelayedVal;
use super::{DelayedParser, DelayedValSet};
use crate::parser::ParseWithResult;
use crate::{AnnotationMode, Parser, ParserSpec};

/// A parser whos output can be referenced before it has been executed
pub struct Delayed<I, O> {
    inner: I,
    /// This will be populated / overwritten whenever the parser is ran.
    value: DelayedVal<O>,
}

impl<I, O> Delayed<I, O> {
    pub fn new<Input>(inner: I) -> Self
    where
        I: Parser<Input>,
    {
        Self {
            inner,
            value: DelayedVal::default(),
        }
    }
}

impl<Input, I> Parser<Input> for Delayed<I, I::Output>
where
    I: Parser<Input>,
{
    type Output = DelayedVal<I::Output>;

    fn name(&self) -> String {
        self.inner.name()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec()
    }

    #[inline(always)]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let (out, anno) = self.inner.parse_with(input, annotation_mode)?;

        // Set the shared value
        self.value.set(out);

        Ok((self.value.clone(), anno))
    }
}

impl<Input, P> DelayedParser<Input> for Delayed<P, P::Output>
where
    P: Parser<Input>,
{
    type Value = P::Output;
    type DelayedValue = Self::Output;

    fn output(&self) -> Self::DelayedValue {
        self.value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ByteParser, ParserAdapter};

    #[test]
    fn test_delayed() {
        fn create_parser() -> impl for<'a> Parser<&'a [u8], Output = DelayedVal<u8>> {
            u8::LE.delay()
        }

        fn use_parser() -> (Vec<u8>, DelayedVal<u8>) {
            let mut parser = create_parser();

            let input = vec![0; 5];
            let (value, _) = parser.parse(&mut input.as_slice()).unwrap();

            (input, value)
        }

        use_parser();
    }
}
