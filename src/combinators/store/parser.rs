use super::{ForwardRef, ForwrdRefSet, StoringParser};
use crate::parser::ParseWithResult;
use crate::{AnnotationMode, Parser, ParserSpec};

/// A parser whos output can be referenced before it has been executed
pub struct Store<I, O> {
    inner: I,
    /// This will be populated / overwritten whenever the parser is ran.
    value: ForwardRef<O>,
}

impl<I, O: 'static> Store<I, O> {
    pub fn new<Input>(inner: I) -> Self
    where
        I: Parser<Input>,
    {
        Self {
            inner,
            value: ForwardRef::new_source(),
        }
    }
}

impl<Input, I> Parser<Input> for Store<I, I::Output>
where
    I: Parser<Input>,
{
    type Output = ForwardRef<I::Output>;

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
        let (out, anno) = self.inner.parse_with(input, annotation_mode)?;

        // Set the shared value
        self.value.set(out);

        Ok((self.value.clone(), anno))
    }
}

impl<Input, P> StoringParser<Input> for Store<P, P::Output>
where
    P: Parser<Input>,
{
    type Value = P::Output;
    type Ref = Self::Output;

    fn output(&self) -> Self::Ref {
        self.value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ByteParser, ParserAdapter};

    #[test]
    fn test_delayed() {
        fn create_parser() -> impl for<'a> Parser<&'a [u8], Output = ForwardRef<u8>> {
            u8::LE.store()
        }

        fn use_parser() -> (Vec<u8>, ForwardRef<u8>) {
            let mut parser = create_parser();

            let input = vec![0; 5];
            let (value, _) = parser.parse(&mut input.as_slice()).unwrap();

            (input, value)
        }

        use_parser();
    }
}
