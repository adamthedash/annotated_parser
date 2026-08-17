use std::cmp::Ordering;

use num_traits::AsPrimitive;

use super::{SkipArray, SkipVec};
use crate::{Annotation, AnnotationReturn, ForwardRefGet, ParseWithResult, Parser, ParserSpec};

impl<const N: usize> Parser<&str> for SkipArray<N> {
    type Output = ();

    #[inline]
    fn name(&self) -> String {
        format!("skip({N})")
    }

    #[inline]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<&str>::name(self))
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let end = match input.chars().count().cmp(&N) {
            Ordering::Less => {
                let annotation = if annotation_mode.fail {
                    Annotation::incomplete(Parser::<&str>::name(self), 0, vec![]).into()
                } else {
                    AnnotationReturn::Start(0)
                };

                return Err(annotation);
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

        *input = &input[end..];

        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<&str>::name(self), 0..N, (), vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok(((), annotation))
    }
}

impl<C> Parser<&str> for SkipVec<C>
where
    C: ForwardRefGet,
    C::Value: AsPrimitive<usize>,
{
    type Output = ();

    #[inline]
    fn name(&self) -> String {
        "skip".to_owned()
    }

    #[inline]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<&str>::name(self))
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let count = self.0.get().as_();

        let end = match input.chars().count().cmp(&count) {
            Ordering::Less => {
                let annotation = if annotation_mode.fail {
                    Annotation::incomplete(Parser::<&str>::name(self), 0, vec![]).into()
                } else {
                    AnnotationReturn::Start(0)
                };

                return Err(annotation);
            }
            Ordering::Equal => input.len(),
            Ordering::Greater => {
                let (end, _) = input
                    .char_indices()
                    .nth(count)
                    .expect("length verified by match predicate");
                end
            }
        };

        *input = &input[end..];

        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<&str>::name(self), 0..count, (), vec![]).into()
        } else {
            AnnotationReturn::Span(0..count)
        };

        Ok(((), annotation))
    }
}
