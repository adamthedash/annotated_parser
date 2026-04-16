use std::fmt::Debug;

use crate::{
    AnnotatedResult, Annotation, Parser, ParserSpec,
    combinators::{Checkpoint, Peek},
    helpers::fold_success,
};

/// Keep taking bytes until the inner parser succeeds
/// On success, input is moved to the start of where the inner parser has succeeded
/// This parser will fail if EOF is reached before the parser succeeds
pub struct TakeTillExc<P> {
    inner: Peek<P>,
}

impl<P> TakeTillExc<P> {
    pub fn new<Input>(inner: P) -> Self
    where
        P: Parser<Input>,
        Input: Copy,
    {
        Self {
            inner: Peek::new(inner),
        }
    }
}

impl<P> Parser<&[u8]> for TakeTillExc<P>
where
    P: for<'a> Parser<&'a [u8]>,
{
    type Output = Vec<u8>;

    fn name(&self) -> String {
        "take_till_exc".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let mut bytes = vec![];

        // TODO: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.annotate(input).is_err() {
            // Advance one byte
            let Some((byte, rest)) = input.split_first() else {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            };

            bytes.push(*byte);
            *input = rest;
        }

        let annotation = Annotation::success(self.name(), 0..bytes.len(), bytes.clone(), vec![]);

        Ok((bytes, annotation))
    }

    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        // TODO: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.annotate(input).is_err() {
            if end == input.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one byte
            end += 1;
            *input = &input[1..];
        }

        Ok((original[..end].to_vec(), end))
    }
}

/// Keep taking bytes until the inner parser succeeds
/// On success, input is moved to the end of where the inner parser has succeeded, and both
/// preceeding tokens and the output of the inner parser are returned.
/// This parser will fail if EOF is reached before the parser succeeds
pub struct TakeTillInc<P> {
    inner: Checkpoint<P>,
}

impl<P> TakeTillInc<P> {
    pub fn new<Input>(inner: P) -> Self
    where
        P: Parser<Input>,
        Input: Copy,
    {
        Self {
            inner: Checkpoint::new(inner),
        }
    }
}

impl<P, PO> Parser<&[u8]> for TakeTillInc<P>
where
    P: for<'a> Parser<&'a [u8], Output = PO>,
    PO: Clone + Debug + 'static,
{
    type Output = (Vec<u8>, PO);

    fn name(&self) -> String {
        "take_till_inc".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        let (value, offset, child_annotations) = loop {
            if let Ok((value, annotation)) = self.inner.annotate(input) {
                let (offset, child_annotations) = fold_success(annotation, vec![], end, 0);
                break (value, offset, child_annotations);
            }

            if end == input.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one byte
            end += 1;
            *input = &input[1..];
        };

        let bytes = original[..end].to_vec();

        let annotation = Annotation::success(
            self.name(),
            0..offset,
            (bytes.clone(), value.clone()),
            child_annotations,
        );

        Ok(((bytes, value), annotation))
    }

    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        let (value, offset) = loop {
            if let Ok((value, offset)) = self.inner.parse(input) {
                break (value, offset);
            }

            if end == input.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one byte
            end += 1;
            *input = &input[1..];
        };

        let bytes = original[..end].to_vec();

        Ok(((bytes, value), offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ByteParser;
    use crate::adapter::ParserAdapter;

    #[test]
    fn test() {
        fn create_parser() -> impl for<'a> Parser<&'a [u8], Output = u8> {
            u8::LE.verify(|x| *x == 0)
        }

        fn use_parser() -> (Vec<u8>, Vec<u8>) {
            let mut parser = TakeTillExc::new(create_parser());

            let input = vec![0; 5];
            let (value, _) = parser.parse(&mut input.as_slice()).unwrap();

            (input, value)
        }

        use_parser();
    }

    #[test]
    fn test_inc() {
        let mut input = b"aaaaabb".as_slice();
        let mut parser = TakeTillInc::new(b"b");

        let ((bytes, value), _) = parser.parse(&mut input).unwrap();
        assert_eq!(bytes, b"aaaaa");
        assert_eq!(value, b"b");
        assert_eq!(input, b"b");
    }

    #[test]
    fn test_esc() {
        let mut input = b"aaaaabb".as_slice();
        let mut parser = TakeTillExc::new(b"b");

        let (bytes, _) = parser.parse(&mut input).unwrap();
        assert_eq!(bytes, b"aaaaa");
        assert_eq!(input, b"bb");
    }
}
