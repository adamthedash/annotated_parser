use crate::{
    Annotation, AnnotationReturn, ForwardRefGet, ParseWithResult, Parser, ParserSpec, input::Input,
};
use num_traits::AsPrimitive;

/// Take a fixed number of elements into an array.
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

impl<I: Input, const N: usize> Parser<I> for TakeArray<N> {
    type Output = I::OwnedConst<N>;

    #[inline]
    fn name(&self) -> String {
        format!("take({N})")
    }

    #[inline]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<I>::name(self))
    }

    fn parse_with(
        &mut self,
        input: &mut I,
        annotation_mode: crate::AnnotationMode,
    ) -> crate::ParseWithResult<Self::Output> {
        let Some((value, rest)) = input.take_const() else {
            let annotation = if annotation_mode.fail {
                Annotation::incomplete(Parser::<I>::name(self), 0, vec![]).into()
            } else {
                AnnotationReturn::Start(0)
            };

            return Err(annotation);
        };

        *input = rest;

        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<I>::name(self), 0..N, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok((value, annotation))
    }
}

/// Take a dynamic number of bytes into a `Vec<u8>`.
/// Fails if the input is shorter than the requested count.
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

impl<I, C> Parser<I> for TakeVec<C>
where
    I: Input,
    C: ForwardRefGet,
    C::Value: AsPrimitive<usize>,
{
    type Output = I::OwnedVar;

    #[inline]
    fn name(&self) -> String {
        "take".to_owned()
    }

    #[inline]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<I>::name(self))
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut I,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let count = self.0.get().as_();

        let Some((value, rest)) = input.take_var(count) else {
            let annotation = if annotation_mode.fail {
                Annotation::incomplete(Parser::<I>::name(self), 0, vec![]).into()
            } else {
                AnnotationReturn::Start(0)
            };
            return Err(annotation);
        };

        *input = rest;

        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<I>::name(self), 0..count, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..count)
        };

        Ok((value, annotation))
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
