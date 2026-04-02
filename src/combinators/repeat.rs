use num_traits::AsPrimitive;

use crate::{FoldResult, FoldSpeedyResult, combinators::delayed::DelayedValGet};
use std::marker::PhantomData;

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

    fn name(&self) -> String {
        format!("repeat({})", N)
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let (values, offset, child_annotations) = (0..N).try_fold(
            (vec![], 0, vec![]),
            |(mut values, offset, child_annotations), _| {
                let (value, span, child_annotations) =
                    self.inner
                        .parse(input)
                        .fold(child_annotations, offset, &self.name(), 0)?;

                values.push(value);

                Ok((values, span.end, child_annotations))
            },
        )?;

        let values = values
            .try_into()
            .expect("Parser should have successfully applied N times above");

        let annotation = Annotation::success(&self.name(), 0..offset, &values, child_annotations);

        Ok((values, annotation))
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        // TODO: Allocate directly on the stack with MaybeUninit
        let (values, offset) = (0..N).try_fold((vec![], 0), |(mut values, offset), _| {
            let (value, offset) = self
                .inner
                .parse_speedy(input)
                .fold(offset, &self.name(), 0)?;

            values.push(value);

            Ok((values, offset))
        })?;

        let values = values
            .try_into()
            .expect("Parser should have successfully applied N times above");

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

    fn name(&self) -> String {
        "repeat".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let count = self.count.get().as_();
        let (values, offset, child_annotations) = (0..count).try_fold(
            (vec![], 0, vec![]),
            |(mut values, offset, child_annotations), _| {
                let (value, span, child_annotations) =
                    self.inner
                        .parse(input)
                        .fold(child_annotations, offset, &self.name(), 0)?;

                values.push(value);

                Ok((values, span.end, child_annotations))
            },
        )?;

        let annotation = Annotation::success(&self.name(), 0..offset, &values, child_annotations);

        Ok((values, annotation))
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        let count = self.count.get().as_();
        let (values, offset) = (0..count).try_fold((vec![], 0), |(mut values, offset), _| {
            let (value, offset) = self
                .inner
                .parse_speedy(input)
                .fold(offset, &self.name(), 0)?;

            values.push(value);

            Ok((values, offset))
        })?;

        Ok((values, offset))
    }
}
