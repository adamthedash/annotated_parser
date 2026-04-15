use super::value::DelayedVal;
use super::{DelayedParser, DelayedValSet};
use crate::{AnnotatedResult, Parser, ParserSpec};

/// A parser whos output can be referenced before it has been executed
pub struct Delayed<I, O> {
    inner: I,
    /// This will be populated / overwritten whenever the parser is ran.
    value: DelayedVal<O>,
}

impl<'a, I> Delayed<I, I::Output>
where
    I: Parser<'a>,
{
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            value: DelayedVal::default(),
        }
    }
}

impl<'a, I> Parser<'a> for Delayed<I, I::Output>
where
    I: Parser<'a>,
{
    type Input = I::Input;
    type Output = DelayedVal<I::Output>;

    fn name(&self) -> String {
        self.inner.name()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec()
    }

    fn annotate(&mut self, input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
        let (out, anno) = self.inner.annotate(input)?;

        // Set the shared value
        self.value.set(out);

        Ok((self.value.clone(), anno))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut Self::Input) -> crate::ParseResult<Self::Output> {
        let (out, offset) = self.inner.parse(input)?;

        // Set the shared value
        self.value.set(out);

        Ok((self.value.clone(), offset))
    }
}

impl<'a, P> DelayedParser<'a> for Delayed<P, P::Output>
where
    P: Parser<'a>,
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
        fn create_parser() -> impl for<'a> Parser<'a, Input = &'a [u8], Output = DelayedVal<u8>> {
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
