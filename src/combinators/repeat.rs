use num_traits::AsPrimitive;

use crate::{FoldResult, combinators::delayed::DelayedValGet, helpers::fold_child_err};
use std::{marker::PhantomData, mem::MaybeUninit};

use crate::{Annotation, Parser, ParserSpec, Result};

/// Compile-time repeat
pub struct RepeatArray<P, O> {
    inner: P,
    _output: PhantomData<O>,
}

impl<const N: usize, P> RepeatArray<P, [P::Output; N]>
where
    P: Parser,
{
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            _output: PhantomData,
        }
    }
}

impl<const N: usize, P> Parser for RepeatArray<P, [P::Output; N]>
where
    P: Parser,
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

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let name = self.name();

        let mut values = [const { MaybeUninit::<P::Output>::uninit() }; N];
        let mut child_annotations = Vec::with_capacity(N);

        let mut offset = 0;
        for (i, value_out) in values.iter_mut().enumerate() {
            match self
                .inner
                .parse(input)
                .fold(child_annotations, offset, &name, 0)
            {
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

        let annotation = Annotation::success(name, 0..offset, values.clone(), child_annotations);

        Ok((values, annotation))
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        let mut values = [const { MaybeUninit::<P::Output>::uninit() }; N];

        let mut offset = 0;
        for (i, value_out) in values.iter_mut().enumerate() {
            match self.inner.parse_speedy(input) {
                Ok((value, new_offset)) => {
                    value_out.write(value);
                    offset = new_offset;
                }
                Err(a) => {
                    // Need to manually drop everything allocated up to now, otherwise we will leak
                    // memory
                    for value in values[..i].iter_mut() {
                        // SAFETY: All values up until this one have been populated by the parser
                        unsafe {
                            value.assume_init_drop();
                        }
                    }

                    let annotation = fold_child_err(a, vec![], offset, self.name(), 0);
                    return Err(annotation);
                }
            }
        }

        // SAFETY: All values have been populated by the parser, or the function has exited
        // Ideally could use MaybeUninit::array_assume_init, but we are on stable
        let values = values.map(|v| unsafe { v.assume_init() });

        Ok((values, offset))
    }
}

/// Runtime repeat
pub struct RepeatVec<P, C> {
    inner: P,
    count: C,
}

impl<P, C, V> RepeatVec<P, C>
where
    P: Parser,
    C: DelayedValGet<Value = V>,
    V: AsPrimitive<usize>,
{
    pub fn new(inner: P, count: C) -> Self {
        Self { inner, count }
    }
}

impl<P, C, V> Parser for RepeatVec<P, C>
where
    P: Parser,
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

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let name = self.name();

        let count = self.count.get().as_();

        let mut child_annotations = Vec::with_capacity(count);
        let mut values = Vec::with_capacity(count);
        let mut offset = 0;
        for _ in 0..count {
            let value;
            (value, offset, child_annotations) =
                self.inner
                    .parse(input)
                    .fold(child_annotations, offset, &name, 0)?;

            values.push(value);
        }

        let annotation = Annotation::success(name, 0..offset, values.clone(), child_annotations);

        Ok((values, annotation))
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        let count = self.count.get().as_();

        let mut values = Vec::with_capacity(count);
        let mut offset = 0;
        for _ in 0..count {
            match self.inner.parse_speedy(input) {
                Ok((value, new_offset)) => {
                    values.push(value);
                    offset = new_offset;
                }
                Err(a) => {
                    let annotation = fold_child_err(a, vec![], offset, self.name(), 0);
                    return Err(annotation);
                }
            }
        }

        Ok((values, offset))
    }
}
