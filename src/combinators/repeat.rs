use crate::{FoldResult, combinators::delayed::DelayedValGet};
use std::{fmt::Debug, marker::PhantomData};

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
    P::Output: Debug,
{
    type Output = [P::Output; N];

    fn name(&self) -> String {
        format!("repeat({})", N)
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let (offset, values, child_annotations) = (0..N).try_fold(
            (0, vec![], vec![]),
            |(offset, mut values, child_annotations), _| {
                let (value, span, child_annotations) =
                    self.inner
                        .parse(input)
                        .fold(child_annotations, offset, &self.name(), 0)?;

                values.push(value);

                Ok((span.end, values, child_annotations))
            },
        )?;

        let values = values
            .try_into()
            .expect("Parser should have successfully applied N times above");

        let annotation = Annotation::success(&self.name(), 0..offset, &values, child_annotations);

        Ok((values, annotation))
    }
}

/// Runtime repeat
pub struct RepeatVec<P, C> {
    inner: P,
    count: C,
}

impl<P, C> RepeatVec<P, C>
where
    P: Parser,
    C: DelayedValGet<Value = usize>,
{
    pub fn new(inner: P, count: C) -> Self {
        Self { inner, count }
    }
}

impl<P, C> Parser for RepeatVec<P, C>
where
    P: Parser,
    P::Output: Debug,
    C: DelayedValGet<Value = usize>,
{
    type Output = Vec<P::Output>;

    fn name(&self) -> String {
        "repeat".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let count = *self.count.get();
        let (offset, values, child_annotations) = (0..count).try_fold(
            (0, vec![], vec![]),
            |(offset, mut values, child_annotations), _| {
                let (value, span, child_annotations) =
                    self.inner
                        .parse(input)
                        .fold(child_annotations, offset, &self.name(), 0)?;

                values.push(value);

                Ok((span.end, values, child_annotations))
            },
        )?;

        let annotation = Annotation::success(&self.name(), 0..offset, &values, child_annotations);

        Ok((values, annotation))
    }
}
