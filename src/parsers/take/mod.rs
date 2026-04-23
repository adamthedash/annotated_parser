mod byte;
mod str;

use crate::combinators::store::ForwardRefGet;
use num_traits::AsPrimitive;

/// Take a fixed amount of bytes into an array
pub struct TakeArray<const N: usize>;

/// Take an amount of bytes into a Vec
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
