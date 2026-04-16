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
        let original = *input;
        let mut end = 0;

        // PERF: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.annotate(input).is_err() {
            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one byte
            end += 1;
            *input = &input[1..];
        }

        let bytes = original[..end].to_vec();

        let annotation = Annotation::success(self.name(), 0..bytes.len(), bytes.clone(), vec![]);

        Ok((bytes, annotation))
    }

    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        // PERF: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.annotate(input).is_err() {
            if end == original.len() {
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

impl<P> Parser<&str> for TakeTillExc<P>
where
    P: for<'a> Parser<&'a str>,
{
    type Output = String;

    fn name(&self) -> String {
        "take_till_exc".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut &str) -> AnnotatedResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        // PERF: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.annotate(input).is_err() {
            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one char
            end += input
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(input.len());

            *input = &original[end..];
        }

        let taken = original[..end].to_string();
        let taken_chars = taken.chars().count();

        let annotation = Annotation::success(self.name(), 0..taken_chars, taken.clone(), vec![]);

        Ok((taken, annotation))
    }

    fn parse(&mut self, input: &mut &str) -> crate::ParseResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        // PERF: Could increase perf a bit by detecting EOF from inner parser
        while self.inner.annotate(input).is_err() {
            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one char
            end += input
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(input.len());

            *input = &original[end..];
        }

        let taken = original[..end].to_string();
        let taken_chars = taken.chars().count();

        Ok((taken, taken_chars))
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

            if end == original.len() {
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

            if end == original.len() {
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

impl<P, PO> Parser<&str> for TakeTillInc<P>
where
    P: for<'a> Parser<&'a str, Output = PO>,
    PO: Clone + Debug + 'static,
{
    type Output = (String, PO);

    fn name(&self) -> String {
        "take_till_inc".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut &str) -> AnnotatedResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        let (value, offset, child_annotations) = loop {
            if let Ok((value, annotation)) = self.inner.annotate(input) {
                let (offset, child_annotations) = fold_success(annotation, vec![], end, 0);
                break (value, offset, child_annotations);
            }

            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one char
            end += input
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(input.len());

            *input = &original[end..];
        };

        let taken = original[..end].to_string();

        let annotation = Annotation::success(
            self.name(),
            0..offset,
            (taken.clone(), value.clone()),
            child_annotations,
        );

        Ok(((taken, value), annotation))
    }

    fn parse(&mut self, input: &mut &str) -> crate::ParseResult<Self::Output> {
        let original = *input;
        let mut end = 0;

        let (value, offset) = loop {
            if let Ok((value, offset)) = self.inner.parse(input) {
                break (value, offset);
            }

            if end == original.len() {
                // EoF
                return Err(Annotation::incomplete(self.name(), 0, vec![]));
            }

            // Advance one char
            end += input
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(input.len());

            *input = &original[end..];
        };

        let taken = original[..end].to_string();

        Ok(((taken, value), offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ByteParser;
    use crate::adapter::ParserAdapter;

    mod byte {
        use super::*;

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
        fn test_exc() {
            let mut input = b"aaaaabb".as_slice();
            let mut parser = TakeTillExc::new(b"b");

            let (bytes, _) = parser.parse(&mut input).unwrap();
            assert_eq!(bytes, b"aaaaa");
            assert_eq!(input, b"bb");
        }
    }

    mod str {
        use super::*;

        #[test]
        fn test_inc() {
            let mut input = "aaaaabb";
            let mut parser = TakeTillInc::new("b");

            let ((bytes, value), _) = parser.parse(&mut input).unwrap();
            assert_eq!(bytes, "aaaaa");
            assert_eq!(value, "b");
            assert_eq!(input, "b");
        }

        #[test]
        fn test_exc() {
            let mut input = "aaaaabb";
            let mut parser = TakeTillExc::new("b");

            let (bytes, _) = parser.parse(&mut input).unwrap();
            assert_eq!(bytes, "aaaaa");
            assert_eq!(input, "bb");
        }
    }
}
