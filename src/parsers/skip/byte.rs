use num_traits::AsPrimitive;

use super::{SkipArray, SkipVec};
use crate::{Annotation, AnnotationReturn, ForwardRefGet, ParseWithResult, Parser, ParserSpec};

impl<const N: usize> Parser<&[u8]> for SkipArray<N> {
    type Output = ();

    #[inline]
    fn name(&self) -> String {
        format!("skip({N})")
    }

    #[inline]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<&[u8]>::name(self))
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        if input.len() < N {
            let annotation = if annotation_mode.fail {
                Annotation::incomplete(Parser::<&[u8]>::name(self), 0, vec![]).into()
            } else {
                AnnotationReturn::Start(0)
            };
            return Err(annotation);
        }

        *input = &input[N..];

        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<&[u8]>::name(self), 0..N, (), vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok(((), annotation))
    }
}

impl<C> Parser<&[u8]> for SkipVec<C>
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
        ParserSpec::empty(Parser::<&[u8]>::name(self))
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let count = self.0.get().as_();

        if input.len() < count {
            let annotation = if annotation_mode.fail {
                Annotation::incomplete(Parser::<&[u8]>::name(self), 0, vec![]).into()
            } else {
                AnnotationReturn::Start(0)
            };
            return Err(annotation);
        }

        *input = &input[count..];

        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<&[u8]>::name(self), 0..count, (), vec![]).into()
        } else {
            AnnotationReturn::Span(0..count)
        };

        Ok(((), annotation))
    }
}
