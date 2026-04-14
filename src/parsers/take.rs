use crate::combinators::delayed::DelayedValGet;
use num_traits::AsPrimitive;

use crate::{AnnotatedResult, Annotation, Parser, ParserSpec};

/// Take a fixed amount of bytes into an array
pub struct TakeArray<const N: usize>;

impl<const N: usize> Parser for TakeArray<N> {
    type Output = [u8; N];

    #[inline(always)]
    fn name(&self) -> String {
        format!("take({})", N)
    }

    #[inline(always)]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let Some((value, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(self.name(), 0, vec![]));
        };

        *input = rest;

        let annotation = Annotation::success(self.name(), 0..N, *value, vec![]);

        Ok((*value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let Some((value, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(self.name(), 0, vec![]));
        };

        *input = rest;

        Ok((*value, N))
    }
}

/// Take an amount of bytes into a Vec
pub struct TakeVec<D>(pub D)
where
    D: DelayedValGet,
    D::Value: AsPrimitive<usize>;

impl<D> Parser for TakeVec<D>
where
    D: DelayedValGet,
    D::Value: AsPrimitive<usize>,
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
