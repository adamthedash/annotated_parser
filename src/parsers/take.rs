use std::cmp::Ordering;

use crate::combinators::delayed::DelayedValGet;
use num_traits::AsPrimitive;

use crate::{AnnotatedResult, Annotation, Parser, ParserSpec};

/// Take a fixed amount of bytes into an array
pub struct TakeArray<const N: usize>;

impl<const N: usize> Parser<&[u8]> for TakeArray<N> {
    type Output = [u8; N];

    #[inline(always)]
    fn name(&self) -> String {
        format!("take({})", N)
    }

    #[inline(always)]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<&[u8]>::name(self))
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let Some((value, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(
                Parser::<&[u8]>::name(self),
                0,
                vec![],
            ));
        };

        *input = rest;

        let annotation = Annotation::success(Parser::<&[u8]>::name(self), 0..N, *value, vec![]);

        Ok((*value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let Some((value, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(
                Parser::<&[u8]>::name(self),
                0,
                vec![],
            ));
        };

        *input = rest;

        Ok((*value, N))
    }
}

impl<const N: usize> Parser<&str> for TakeArray<N> {
    type Output = String;

    #[inline(always)]
    fn name(&self) -> String {
        format!("take({})", N)
    }

    #[inline(always)]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<&str>::name(self))
    }

    fn annotate(&mut self, input: &mut &str) -> AnnotatedResult<Self::Output> {
        let end = match input.chars().count().cmp(&N) {
            Ordering::Less => {
                return Err(Annotation::incomplete(
                    Parser::<&str>::name(self),
                    0,
                    vec![],
                ));
            }
            Ordering::Equal => input.len(),
            Ordering::Greater => {
                let (end, _) = input
                    .char_indices()
                    .nth(N)
                    .expect("length verified by match predicate");
                end
            }
        };

        let value = input[..end].to_string();

        *input = &input[end..];

        let annotation =
            Annotation::success(Parser::<&str>::name(self), 0..N, value.clone(), vec![]);

        Ok((value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut &str) -> crate::ParseResult<Self::Output> {
        let end = match input.chars().count().cmp(&N) {
            Ordering::Less => {
                return Err(Annotation::incomplete(
                    Parser::<&str>::name(self),
                    0,
                    vec![],
                ));
            }
            Ordering::Equal => input.len(),
            Ordering::Greater => {
                let (end, _) = input
                    .char_indices()
                    .nth(N)
                    .expect("length verified by match predicate");
                end
            }
        };

        let value = input[..end].to_string();

        *input = &input[end..];

        Ok((value, N))
    }
}

/// Take an amount of bytes into a Vec
pub struct TakeVec<C>(C);

impl<C> TakeVec<C> {
    pub fn new(count: C) -> Self
    where
        C: DelayedValGet,
        C::Value: AsPrimitive<usize>,
    {
        Self(count)
    }
}

impl<C> Parser<&[u8]> for TakeVec<C>
where
    C: DelayedValGet,
    C::Value: AsPrimitive<usize>,
{
    type Output = Vec<u8>;

    #[inline(always)]
    fn name(&self) -> String {
        "take".to_owned()
    }

    #[inline(always)]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let count = self.0.get().as_();

        let Some((value, rest)) = input.split_at_checked(count) else {
            return Err(Annotation::incomplete(self.name(), 0, vec![]));
        };
        let value = value.to_vec();

        *input = rest;

        let annotation = Annotation::success(self.name(), 0..count, value.clone(), vec![]);

        Ok((value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let count = self.0.get().as_();

        let Some((value, rest)) = input.split_at_checked(count) else {
            return Err(Annotation::incomplete(self.name(), 0, vec![]));
        };

        *input = rest;

        Ok((value.to_vec(), count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
