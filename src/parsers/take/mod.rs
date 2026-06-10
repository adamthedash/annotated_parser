mod byte;
mod str;

use crate::ForwardRefGet;
use num_traits::AsPrimitive;

/// Take a fixed number of elements into an array.
///
/// For `&[u8]` inputs, consumes `N` bytes and returns `[u8; N]`.
/// For `&str` inputs, consumes `N` characters and returns a `String`.
/// Fails if the input is too short.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::TakeArray;
///
/// let mut input = "hello";
/// let (value, _) = TakeArray::<3>.parse(&mut input).unwrap();
/// assert_eq!(value, "hel");
/// assert_eq!(input, "lo");
/// ```
pub struct TakeArray<const N: usize>;

/// Take a dynamic number of bytes into a `Vec<u8>`.
///
/// The count is determined by the value produced by the inner `C` parser
/// at parse time. Fails if the input is shorter than the requested count.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::TakeVec;
/// use annotated_parser::ForwardRef;
///
/// let count = ForwardRef::with_value(3usize);
/// let mut input = &[1, 2, 3, 4][..];
/// let (value, _) = TakeVec::new(count).parse(&mut input).unwrap();
/// assert_eq!(value, vec![1, 2, 3]);
/// assert_eq!(input, &[4]);
/// ```
pub struct TakeVec<C>(C);

impl<C> TakeVec<C> {
    pub fn new(count: C) -> Self
    where
        C: ForwardRefGet,
        C::Value: AsPrimitive<usize>,
    {
        Self(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_str() {
        let mut input = "hello";
        let mut parser = TakeArray::<3>;

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, "hel");
        assert_eq!(input, "lo");
    }

    #[test]
    fn test_str_all() {
        let mut input = "hello";
        let mut parser = TakeArray::<5>;

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, "hello");
        assert_eq!(input, "");
    }

    #[test]
    fn test_str_short() {
        let mut input = "hello";
        let mut parser = TakeArray::<7>;

        let anno = parser.parse(&mut input).unwrap_err();
        assert_eq!(input, "hello");
        assert!(!anno.result.is_ok());
    }
}
