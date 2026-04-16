use std::{marker::PhantomData, mem::MaybeUninit};

use crate::{Annotation, FoldResult, Parser, ParserSpec};

pub struct SeparatedArray<P, S, O> {
    separator: S,
    inner: P,
    // Needed to constrain N
    _output: PhantomData<O>,
}

impl<const N: usize, P, S, O> SeparatedArray<P, S, [O; N]> {
    pub fn new<Input>(separator: S, inner: P) -> Self
    where
        S: Parser<Input>,
        P: Parser<Input>,
    {
        Self {
            separator,
            inner,
            _output: PhantomData,
        }
    }
}

impl<const N: usize, Input, P, S> Parser<Input> for SeparatedArray<P, S, [P::Output; N]>
where
    S: Parser<Input>,
    P: Parser<Input>,
{
    type Output = [P::Output; N];

    fn name(&self) -> String {
        format!("separated({N})")
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::new(self.name(), vec![self.separator.spec(), self.inner.spec()])
    }

    fn annotate(&mut self, input: &mut Input) -> crate::AnnotatedResult<Self::Output> {
        let mut values = [const { MaybeUninit::<P::Output>::uninit() }; N];
        let mut child_annotations = Vec::with_capacity(N);

        let mut offset = 0;
        for (i, value_out) in values.iter_mut().enumerate() {
            // Separator
            if i > 0 {
                match self.separator.annotate(input).fold(
                    child_annotations,
                    offset,
                    || self.name(),
                    0,
                ) {
                    Ok((_ignored, new_offset, child_annos)) => {
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

            // Inner
            match self
                .inner
                .annotate(input)
                .fold(child_annotations, offset, || self.name(), 1)
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

        let annotation =
            Annotation::success(self.name(), 0..offset, values.clone(), child_annotations);

        Ok((values, annotation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good() {
        let mut input = "A A A A A";
        let mut parser = SeparatedArray::new(" ", "A");

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, ["A"; 5]);
        assert_eq!(input, "");
    }

    #[test]
    fn test_empty() {
        let mut input = "";
        let mut parser = SeparatedArray::new(" ", "A");

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, ["A"; 0]);
        assert_eq!(input, "");
    }
}
