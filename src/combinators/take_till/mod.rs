mod byte;
mod str;

use crate::{
    Parser,
    combinators::{Checkpoint, Peek},
};

/// Consume input until the inner parser succeeds, stopping at the match point.
///
/// Repeatedly advances the input and peeks at the inner parser.
/// When the inner parser succeeds, stops and returns the consumed prefix.
/// The input is left positioned at the start of the match.
/// Fails if EOF is reached before the inner parser succeeds.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::combinators::TakeTillExc;
///
/// let mut parser = TakeTillExc::new(b"b");
/// let mut input = b"aaaaabb".as_slice();
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, b"aaaaa");
/// assert_eq!(input, b"bb");
/// ```
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

/// Consume input until the inner parser succeeds, including the match.
///
/// Repeatedly advances the input and tries the inner parser.
/// When the inner parser succeeds, stops and returns both the consumed prefix and the match's output.
/// The input is left positioned after the match.
/// Fails if EOF is reached before the inner parser succeeds.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::combinators::TakeTillInc;
///
/// let mut parser = TakeTillInc::new(b"b");
/// let mut input = b"aaaaabb".as_slice();
/// let ((bytes, value), _) = parser.parse(&mut input).unwrap();
/// assert_eq!(bytes, b"aaaaa");
/// assert_eq!(value, b"b");
/// assert_eq!(input, b"b");
/// ```
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
