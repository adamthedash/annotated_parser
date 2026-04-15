use paste::paste;
use std::fmt::Debug;

use crate::{
    AnnotatedResult, Annotation, FoldResult, ParseResult, Parser, ParserSpec,
    combinators::delayed::DelayedValGet, helpers::fold_child_err,
};

/// Helper trait for interacting with tuples of parsers
pub trait ParserTuple<Input> {
    type Output: Debug + Clone + 'static;

    /// Call Parser::spec on all child parsers
    fn specs(&self) -> Vec<ParserSpec>;

    /// Call Parser::annotate on a specific child parser
    fn annotate(
        &mut self,
        input: &mut Input,
        index: usize,
    ) -> Option<AnnotatedResult<Self::Output>>;

    /// Call Parser::parse on a specific child parser
    fn parse(&mut self, input: &mut Input, index: usize) -> Option<ParseResult<Self::Output>>;
}

macro_rules! impl_parser_tuple_for_tuple {
    ( $First:ident ~ $first_idx:tt $(, $P:ident ~ $idx:tt )* ) => {
        paste! {
            impl<Input, $First $(, $P)*> ParserTuple<Input> for ($First, $($P,)*)
            where
                $First: Parser<Input>,
                $(
                    $P: Parser<Input, Output = $First::Output>,
                )*
            {
                type Output = $First::Output;

                #[inline(always)]
                fn specs(&self) -> Vec<ParserSpec> {
                    vec![
                        self.$first_idx.spec(),
                        $( self.$idx.spec(), )*
                    ]
                }

                #[inline(always)]
                fn annotate(&mut self, input: &mut Input, index: usize) -> Option<AnnotatedResult<Self::Output>> {
                    match index {
                        $first_idx => Some(self.$first_idx.annotate(input)),
                        $( $idx => Some(self.$idx.annotate(input)), )*
                        _ => None,
                    }
                }

                #[inline(always)]
                fn parse(&mut self, input: &mut Input, index: usize) -> Option<ParseResult<Self::Output>> {
                    match index {
                        $first_idx => Some(self.$first_idx.parse(input)),
                        $( $idx => Some(self.$idx.parse(input)), )*
                        _ => None,
                    }
                }
            }
        }
    };
}

impl_parser_tuple_for_tuple!(A~0);
impl_parser_tuple_for_tuple!(A~0, B~1);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2, D~3);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2, D~3, E~4);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10);
impl_parser_tuple_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10, L~11);

pub struct Dispatch<D, P> {
    discriminant: D,
    parsers: P,
}

impl<D, P> Dispatch<D, P>
where
    D: DelayedValGet<Value = Option<usize>>,
{
    pub fn new<Input>(discriminant: D, parsers: P) -> Self
    where
        P: ParserTuple<Input>,
    {
        Self {
            discriminant,
            parsers,
        }
    }
}

impl<Input, D, P> Parser<Input> for Dispatch<D, P>
where
    D: DelayedValGet<Value = Option<usize>>,
    P: ParserTuple<Input>,
{
    type Output = P::Output;

    fn name(&self) -> String {
        "dispatch".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), self.parsers.specs())
    }

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        let Some(index) = *self.discriminant.get() else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                "Unknown discriminant".to_string(),
                vec![],
            ));
        };

        let Some(res) = self.parsers.annotate(input, index) else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                "Discriminant out of bounds".to_string(),
                vec![],
            ));
        };

        let (value, offset, child_annotations) = res.fold(vec![], 0, || self.name(), index)?;

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let Some(index) = *self.discriminant.get() else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                "Unknown discriminant".to_string(),
                vec![],
            ));
        };

        let Some(res) = self.parsers.parse(input, index) else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                "Discriminant out of bounds".to_string(),
                vec![],
            ));
        };

        let (value, offset) = res.map_err(|a| fold_child_err(a, vec![], 0, self.name(), index))?;

        Ok((value, offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ByteParser;
    use crate::ParserAdapter;
    use crate::combinators::delayed::DelayedParser;
    use crate::combinators::delayed::DelayedVal;

    #[test]
    fn test_dispatch() {
        let mut input = [0, 1, 1, 1, 0].as_slice();

        let disc_parser = u8::LE.delay();
        let dispatch = Dispatch::new(
            disc_parser.output().map(|x| {
                let index = match x {
                    0 => 0,
                    1 => 1,
                    _ => return None,
                };

                Some(index)
            }),
            (
                u8::LE, //
                u16::LE.map(|x| x as u8),
            ),
        );

        let mut parser = (disc_parser, dispatch).repeat::<2>();
        parser.annotate(&mut input).unwrap();
    }

    #[test]
    fn test_dispatch2() {
        fn create_parser() -> impl for<'a> Parser<&'a [u8], Output = u8> {
            Dispatch::new(
                DelayedVal::with_value(Some(0)),
                (
                    u8::LE, //
                    u16::LE.map(|x| x as u8),
                ),
            )
        }

        fn use_parser() -> (Vec<u8>, u8) {
            let mut parser = create_parser();

            let input = vec![0; 5];
            let (value, _) = parser.parse(&mut input.as_slice()).unwrap();

            (input, value)
        }

        use_parser();
    }
}
