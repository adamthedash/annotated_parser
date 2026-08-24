use crate::{ALLOC_LIMIT, AnnotationReturn, helpers::FoldParseWithResult, parser::ParseWithResult};
use num_traits::AsPrimitive;

use crate::combinators::store::ForwardRefGet;
use std::{marker::PhantomData, mem::MaybeUninit, sync::atomic::Ordering};

use crate::{Annotation, Parser, ParserSpec};

/// Repeat a parser a fixed number of times at compile time.
///
/// Applies the inner parser `N` times and collects the results into an array.
/// Fails if any repetition fails.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::ByteParser;
///
/// let mut parser = u8::LE.repeat::<2>();
/// let mut input = &[1, 2, 3][..];
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, [1, 2]);
/// assert_eq!(input, &[3]);
/// ```
pub struct RepeatArray<P, O> {
    inner: P,
    // Needed to constrain N
    _output: PhantomData<O>,
}

impl<const N: usize, P, O> RepeatArray<P, [O; N]> {
    pub fn new<Input>(inner: P) -> Self
    where
        P: Parser<Input>,
    {
        Self {
            inner,
            _output: PhantomData,
        }
    }
}

impl<const N: usize, Input, P> Parser<Input> for RepeatArray<P, [P::Output; N]>
where
    P: Parser<Input>,
{
    type Output = [P::Output; N];

    #[inline]
    fn name(&self) -> String {
        format!("repeat({})", N)
    }

    #[inline]
    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let mut child_annotations = annotation_mode.success.then(|| Vec::with_capacity(N));

        let mut values = [const { MaybeUninit::<P::Output>::uninit() }; N];
        let mut offset = 0;

        for (i, value_out) in values.iter_mut().enumerate() {
            // Inner
            match self.inner.parse_with(input, annotation_mode).fold(
                annotation_mode,
                || self.name(),
                child_annotations,
                offset,
                0,
            ) {
                Ok((value, new_offset, child_annos)) => {
                    value_out.write(value);
                    offset = new_offset;
                    child_annotations = child_annos;
                }
                Err(annotation) => {
                    // Need to manually drop everything allocated up to now, otherwise we will leak
                    // memory
                    for value in values[..i].iter_mut() {
                        // SAFETY: All values up until this one have been populated by the parser
                        unsafe {
                            value.assume_init_drop();
                        }
                    }

                    return Err(annotation);
                }
            }
        }

        // SAFETY: All values have been populated by the parser, or the function has exited
        // Ideally could use MaybeUninit::array_assume_init, but we are on stable
        let values = values.map(|v| unsafe { v.assume_init() });

        let annotation = if annotation_mode.success {
            Annotation::success(
                self.name(),
                0..offset,
                values.clone(),
                child_annotations.unwrap(),
            )
            .into()
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok((values, annotation))
    }
}

/// Repeat a parser a runtime-determined number of times.
///
/// The count is provided by a `ForwardRefGet` value. Collects results into a `Vec`.
/// Fails if any repetition fails.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::ByteParser;
/// use annotated_parser::ForwardRef;
///
/// let count = ForwardRef::with_value(2usize);
/// let mut parser = u8::LE.repeat_vec(count);
/// let mut input = &[1, 2, 3][..];
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, vec![1, 2]);
/// assert_eq!(input, &[3]);
/// ```
pub struct RepeatVec<P, C> {
    inner: P,
    count: C,
}

impl<P, C> RepeatVec<P, C> {
    pub fn new<Input>(inner: P, count: C) -> Self
    where
        P: Parser<Input>,
        C: ForwardRefGet,
        C::Value: AsPrimitive<usize>,
    {
        Self { inner, count }
    }
}

impl<Input, P, C, V> Parser<Input> for RepeatVec<P, C>
where
    P: Parser<Input>,
    C: ForwardRefGet<Value = V>,
    V: AsPrimitive<usize>,
{
    type Output = Vec<P::Output>;

    #[inline]
    fn name(&self) -> String {
        "repeat".to_owned()
    }

    #[inline]
    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let count = self.count.get().as_();

        // Ensure we don't blow up
        if count > ALLOC_LIMIT.load(Ordering::Relaxed) {
            let annotation = if annotation_mode.fail {
                Annotation::oom(self.name(), 0, count).into()
            } else {
                AnnotationReturn::Start(0)
            };

            return Err(annotation);
        }

        let mut child_annotations = annotation_mode.success.then(|| Vec::with_capacity(count));

        let mut values = Vec::with_capacity(count);
        let mut offset = 0;

        let mut value;
        for _ in 0..count {
            (value, offset, child_annotations) =
                self.inner.parse_with(input, annotation_mode).fold(
                    annotation_mode,
                    || self.name(),
                    child_annotations,
                    offset,
                    0,
                )?;

            values.push(value);
        }

        let annotation = if annotation_mode.success {
            Annotation::success(
                self.name(),
                0..offset,
                values.clone(),
                child_annotations.unwrap(),
            )
            .into()
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok((values, annotation))
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnnotationResult, ForwardRef, prelude::*};

    #[test]
    fn test_repeat_oom() {
        let mut parser = u32::LE.repeat_vec(ForwardRef::with_value(u64::MAX));
        let input = b"123";

        let anno = parser.annotate(&mut input.as_slice()).unwrap_err();

        let AnnotationResult::OOM { start, requested } = anno.result else {
            panic!("expected OOM got {:?}", anno.result);
        };
        assert_eq!(start, 0);
        assert_eq!(requested, u64::MAX as usize);
    }
}
