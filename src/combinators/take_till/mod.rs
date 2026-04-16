mod byte;
mod str;

use crate::{
    Parser,
    combinators::{Checkpoint, Peek},
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
