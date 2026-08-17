use num_traits::AsPrimitive;

use crate::ForwardRefGet;

mod byte;
mod str;

pub struct SkipArray<const N: usize>;

pub struct SkipVec<C>(C);

impl<C> SkipVec<C> {
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
    use crate::{ForwardRef, Parser};

    mod byte {
        use super::*;

        #[test]
        fn skip_partial() {
            let mut input = [1, 2, 3, 4, 5].as_slice();
            let (value, _) = SkipArray::<3>.parse(&mut input).unwrap();
            assert_eq!(value, ());
            assert_eq!(input, [4, 5]);
        }

        #[test]
        fn skip_all() {
            let mut input = [1, 2, 3, 4, 5].as_slice();
            let (value, _) = SkipArray::<5>.parse(&mut input).unwrap();
            assert_eq!(value, ());
            assert_eq!(input, []);
        }

        #[test]
        fn skip_too_short() {
            let mut input = [1, 2, 3, 4, 5].as_slice();
            let annotation = SkipArray::<7>.parse(&mut input).unwrap_err();
            assert!(!annotation.result.is_ok());
            assert_eq!(input, [1, 2, 3, 4, 5]);
        }

        #[test]
        fn skip_zero() {
            let mut input = [1, 2].as_slice();
            let (value, _) = SkipArray::<0>.parse(&mut input).unwrap();
            assert_eq!(value, ());
            assert_eq!(input, [1, 2]);
        }

        #[test]
        fn vec_skip_partial() {
            let mut input = [1, 2, 3, 4, 5].as_slice();
            let (value, _) = SkipVec::new(ForwardRef::with_value(3usize))
                .parse(&mut input)
                .unwrap();
            assert_eq!(value, ());
            assert_eq!(input, [4, 5]);
        }

        #[test]
        fn vec_skip_all() {
            let mut input = [1, 2, 3, 4, 5].as_slice();
            let (value, _) = SkipVec::new(ForwardRef::with_value(5usize))
                .parse(&mut input)
                .unwrap();
            assert_eq!(value, ());
            assert_eq!(input, []);
        }

        #[test]
        fn vec_skip_too_short() {
            let mut input = [1, 2, 3, 4, 5].as_slice();
            let annotation = SkipVec::new(ForwardRef::with_value(7usize))
                .parse(&mut input)
                .unwrap_err();
            assert!(!annotation.result.is_ok());
            assert_eq!(input, [1, 2, 3, 4, 5]);
        }

        #[test]
        fn vec_skip_zero() {
            let mut input = [1, 2].as_slice();
            let (value, _) = SkipVec::new(ForwardRef::with_value(0usize))
                .parse(&mut input)
                .unwrap();
            assert_eq!(value, ());
            assert_eq!(input, [1, 2]);
        }
    }

    mod str {
        use super::*;

        #[test]
        fn skip_partial() {
            let mut input = "hello";
            let (value, _) = SkipArray::<3>.parse(&mut input).unwrap();
            assert_eq!(value, ());
            assert_eq!(input, "lo");
        }

        #[test]
        fn skip_all() {
            let mut input = "hello";
            let (value, _) = SkipArray::<5>.parse(&mut input).unwrap();
            assert_eq!(value, ());
            assert_eq!(input, "");
        }

        #[test]
        fn skip_too_short() {
            let mut input = "hello";
            let annotation = SkipArray::<7>.parse(&mut input).unwrap_err();
            assert!(!annotation.result.is_ok());
            assert_eq!(input, "hello");
        }

        #[test]
        fn skip_zero() {
            let mut input = "ab";
            let (value, _) = SkipArray::<0>.parse(&mut input).unwrap();
            assert_eq!(value, ());
            assert_eq!(input, "ab");
        }

        #[test]
        fn skip_multibyte() {
            let mut input = "αβγ";
            let (value, _) = SkipArray::<2>.parse(&mut input).unwrap();
            assert_eq!(value, ());
            assert_eq!(input, "γ");
        }

        #[test]
        fn vec_skip_partial() {
            let mut input = "hello";
            let (value, _) = SkipVec::new(ForwardRef::with_value(3usize))
                .parse(&mut input)
                .unwrap();
            assert_eq!(value, ());
            assert_eq!(input, "lo");
        }

        #[test]
        fn vec_skip_all() {
            let mut input = "hello";
            let (value, _) = SkipVec::new(ForwardRef::with_value(5usize))
                .parse(&mut input)
                .unwrap();
            assert_eq!(value, ());
            assert_eq!(input, "");
        }

        #[test]
        fn vec_skip_too_short() {
            let mut input = "hello";
            let annotation = SkipVec::new(ForwardRef::with_value(7usize))
                .parse(&mut input)
                .unwrap_err();
            assert!(!annotation.result.is_ok());
            assert_eq!(input, "hello");
        }

        #[test]
        fn vec_skip_zero() {
            let mut input = "ab";
            let (value, _) = SkipVec::new(ForwardRef::with_value(0usize))
                .parse(&mut input)
                .unwrap();
            assert_eq!(value, ());
            assert_eq!(input, "ab");
        }

        #[test]
        fn vec_skip_multibyte() {
            let mut input = "αβγ";
            let (value, _) = SkipVec::new(ForwardRef::with_value(2usize))
                .parse(&mut input)
                .unwrap();
            assert_eq!(value, ());
            assert_eq!(input, "γ");
        }
    }
}
