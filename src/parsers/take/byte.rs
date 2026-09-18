use crate::{AnnotationReturn, ForwardRefGet, ParseWithResult, ParserExec, ParserInfo};
use num_traits::AsPrimitive;

use crate::Annotation;

use super::{TakeArray, TakeVec};

impl<const N: usize> ParserExec<&[u8]> for TakeArray<N> {
    type Output = [u8; N];

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let Some((value, rest)) = input.split_first_chunk() else {
            let annotation = if annotation_mode.fail {
                Annotation::incomplete(self.name(), 0, vec![]).into()
            } else {
                AnnotationReturn::Start(0)
            };
            return Err(annotation);
        };

        *input = rest;

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..N, *value, vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok((*value, annotation))
    }
}

impl<C> ParserExec<&[u8]> for TakeVec<C>
where
    C: ForwardRefGet,
    C::Value: AsPrimitive<usize>,
{
    type Output = Vec<u8>;

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let count = self.0.get().as_();

        let Some((value, rest)) = input.split_at_checked(count) else {
            let annotation = if annotation_mode.fail {
                Annotation::incomplete(self.name(), 0, vec![]).into()
            } else {
                AnnotationReturn::Start(0)
            };
            return Err(annotation);
        };
        let value = value.to_vec();

        *input = rest;

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..count, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..count)
        };

        Ok((value, annotation))
    }
}
