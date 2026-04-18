use crate::AnnotationReturn;
use crate::helpers::FoldParseWithResult;
use crate::parser::ParseWithResult;
use paste::paste;
use std::{marker::PhantomData, mem::MaybeUninit};

use crate::{Annotation, Parser, ParserSpec, combinators::ParserTuple};

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
            // Separator
            if i > 0 {
                match self.separator.parse_with(input, annotation_mode).fold(
                    annotation_mode,
                    || self.name(),
                    child_annotations,
                    offset,
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
            match self.inner.parse_with(input, annotation_mode).fold(
                annotation_mode,
                || self.name(),
                child_annotations,
                offset,
                1,
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

pub struct SeparatedTuple<S, P> {
    separator: S,
    parsers: P,
}

impl<S, P> SeparatedTuple<S, P> {
    pub fn new<Input>(separator: S, parsers: P) -> Self
    where
        S: Parser<Input>,
        P: ParserTuple<Input>,
    {
        Self { separator, parsers }
    }
}

macro_rules! impl_separated_tuple {
    ( $First:ident ~ $first_idx:tt $(, $P:ident ~ $idx:tt )* ) => {
        paste! {
            impl<Input, S, $First $(, $P)*> Parser<Input> for SeparatedTuple<S, ($First $(, $P)*)>
            where
                S: Parser<Input>,
                $First: Parser<Input>,
                $($P: Parser<Input>,)*
            {
                type Output = ($First::Output $(, $P::Output)*);

                fn name(&self) -> String {
                    "separated_tuple".to_owned()
                }

                fn spec(&self) -> ParserSpec {
                    ParserSpec::new(
                        self.name(),
                        std::iter::once(self.separator.spec())
                            .chain(self.parsers.specs())
                            .collect(),
                    )
                }

                #[inline]
                fn parse_with(
                    &mut self,
                    input: &mut Input,
                    annotation_mode: crate::AnnotationMode,
                ) -> ParseWithResult<Self::Output> {
                    let mut child_annotations = annotation_mode.success.then(Vec::new);
                    let mut offset = 0;

                    let [<val_ $first_idx>];
                    ([<val_ $first_idx>], offset, child_annotations) = self.parsers.$first_idx
                        .parse_with(input, annotation_mode).fold(
                            annotation_mode,
                            || self.name(),
                            child_annotations,
                            offset,
                            $first_idx + 1,
                        )?;

                    $(
                        (_, offset, child_annotations) = self.separator
                            .parse_with(input, annotation_mode).fold(
                                annotation_mode,
                                || self.name(),
                                child_annotations,
                                offset,
                                0,
                            )?;

                        let [<val_ $idx>];
                        ([<val_ $idx>], offset, child_annotations) = self.parsers.$idx
                            .parse_with(input, annotation_mode).fold(
                                annotation_mode,
                                || self.name(),
                                child_annotations,
                                offset,
                                $idx + 1,
                            )?;
                    )*

                    let value = ([<val_ $first_idx>] $(, [<val_ $idx>])*);

                    let annotation = if annotation_mode.success {
                        Annotation::success(
                            self.name(),
                            0..offset,
                            value.clone(),
                            child_annotations.unwrap(),
                        )
                        .into()
                    } else {
                        AnnotationReturn::Span(0..offset)
                    };

                    Ok((value, annotation))
                }
            }
        }
    };
}

impl_separated_tuple!(A~0, B~1);
impl_separated_tuple!(A~0, B~1, C~2);
impl_separated_tuple!(A~0, B~1, C~2, D~3);
impl_separated_tuple!(A~0, B~1, C~2, D~3, E~4);
impl_separated_tuple!(A~0, B~1, C~2, D~3, E~4, F~5);
impl_separated_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6);
impl_separated_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7);
impl_separated_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8);
impl_separated_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9);
impl_separated_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10);
impl_separated_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10, L~11);

#[cfg(test)]
mod tests {
    use super::*;

    mod arr {
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

    mod tuple {
        use crate::ParserAdapter;

        use super::*;

        #[test]
        fn test_good() {
            let mut input = "AA B";
            let mut parser = SeparatedTuple::new(" ", (ParserAdapter::repeat::<2>("A"), "B"));

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, (["A", "A"], "B"));
            assert_eq!(input, "");
        }
    }
}
