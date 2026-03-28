use crate::{Annotation, Parser, ParserSpec, Result, combinators::Peek};

/// Keep taking bytes until the inner parser succeeds
/// On success, input is moved to the start of where the inner parser has succeeded
/// This parser will fail if EOF is reached before the parser succeeds
pub struct TakeTill<P> {
    inner: Peek<P>,
}

impl<P> TakeTill<P>
where
    P: Parser,
{
    pub fn new(inner: P) -> Self {
        Self { inner: Peek(inner) }
    }
}

impl<P> Parser for TakeTill<P>
where
    P: Parser,
{
    type Output = Vec<u8>;

    fn name(&self) -> String {
        "take_till".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let mut bytes = vec![];

        // TODO: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.parse(input).is_err() {
            // Advance one byte
            let Some((byte, rest)) = input.split_first() else {
                // EoF
                return Err(Annotation::incomplete(&self.name(), 0, vec![]));
            };

            bytes.push(*byte);
            *input = rest;
        }

        let annotation = Annotation::success(&self.name(), 0..bytes.len(), &bytes, vec![]);

        Ok((bytes, annotation))
    }
}
