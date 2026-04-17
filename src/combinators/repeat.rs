use crate::{AnnotationReturn, helpers::FoldParseWithResult, parser::ParseWithResult};
use num_traits::AsPrimitive;

use crate::combinators::delayed::DelayedValGet;
use std::{marker::PhantomData, mem::MaybeUninit};

use crate::{Annotation, Parser, ParserSpec};

/// Compile-time repeat
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

    #[inline(always)]
    fn name(&self) -> String {
        format!("repeat({})", N)
    }

    #[inline(always)]
    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline(always)]
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

/// Runtime repeat
pub struct RepeatVec<P, C> {
    inner: P,
    count: C,
}

impl<P, C> RepeatVec<P, C> {
    pub fn new<Input>(inner: P, count: C) -> Self
    where
        P: Parser<Input>,
        C: DelayedValGet,
        C::Value: AsPrimitive<usize>,
    {
        Self { inner, count }
    }
}

impl<Input, P, C, V> Parser<Input> for RepeatVec<P, C>
where
    P: Parser<Input>,
    C: DelayedValGet<Value = V>,
    V: AsPrimitive<usize>,
{
    type Output = Vec<P::Output>;

    #[inline(always)]
    fn name(&self) -> String {
        "repeat".to_owned()
    }

    #[inline(always)]
    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    #[inline(always)]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let count = self.count.get().as_();

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
