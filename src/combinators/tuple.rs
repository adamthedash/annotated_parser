use crate::{
    AnnotatedResult, Annotation, FoldAnnotatedResult, ParseResult, Parser, ParserSpec,
    helpers::FoldParseResult,
};
use paste::paste;
use std::fmt::Debug;

/// Tuples of parsers
macro_rules! impl_parser_for_tuple {
    ( $First:ident ~ $first_idx:tt $(, $P:ident ~ $idx:tt )* ) => {
        paste! {
            impl<Input, $First $(, $P)*> Parser<Input> for ($First, $($P,)*)
            where
                $First: Parser<Input>,
                $(
                    $P: Parser<Input>,
                )*
            {

                type Output = ($First::Output, $($P::Output,)*);

                fn name(&self) -> String {
                    "tuple".to_owned()
                }

                fn spec(&self) -> ParserSpec {
                    ParserSpec::new(self.name(), vec![
                        self.$first_idx.spec(),
                        $( self.$idx.spec(), )*
                    ])
                }

                fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
                    let mut child_annotations = vec![];
                    let mut offset = 0usize;

                    let [<out_ $first_idx>];
                    ([<out_ $first_idx>], offset, child_annotations) =
                        self.$first_idx
                            .annotate(input)
                            .fold(child_annotations, offset, || self.name(), $first_idx)?;

                    $(
                        let [<out_ $idx>];
                        ([<out_ $idx>], offset, child_annotations) =
                            self.$idx
                                .annotate(input)
                                .fold(child_annotations, offset, || self.name(), $idx)?;
                    )*

                    let out = ([<out_ $first_idx>], $( [<out_ $idx>], )*);
                    let annotation = Annotation::success(&self.name(), 0..offset, out.clone(), child_annotations);
                    Ok((out, annotation))
                }

                #[inline(always)]
                fn parse(&mut self, input: &mut Input) -> ParseResult<Self::Output> {
                    let mut offset = 0usize;

                    let [<out_ $first_idx>];
                    ([<out_ $first_idx>], offset) =
                        self.$first_idx
                            .parse(input)
                            .fold(offset, || self.name(), $first_idx)?;

                    $(
                        let [<out_ $idx>];
                        ([<out_ $idx>], offset) =
                            self.$idx
                                .parse(input)
                            .fold(offset, || self.name(), $idx)?;
                    )*

                    let out = ([<out_ $first_idx>], $( [<out_ $idx>], )*);
                    Ok((out, offset))
                }
            }
        }
    };
}

// NOTE: Only implemented up to 12-tuples, since Debug is only implemented up to 12.
// If more are needed, just nest the tuples.
impl_parser_for_tuple!(A~0);
impl_parser_for_tuple!(A~0, B~1);
impl_parser_for_tuple!(A~0, B~1, C~2);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10, L~11);

/// Marker trait for tuples of parsers
pub trait ParserTuple<Input> {
    /// Call Parser::spec on all child parsers
    fn specs(&self) -> Vec<ParserSpec>;
}

macro_rules! impl_parser_tuple {
    ( $( $P:ident ~ $idx:tt ),+ ) => {
        impl<Input, $($P),+> ParserTuple<Input> for ($($P,)+)
        where
            $($P: Parser<Input>,)+
        {
            fn specs(&self) -> Vec<ParserSpec> {
                vec![$( self.$idx.spec() ),+]
            }
        }
    };
}

impl_parser_tuple!(A~0);
impl_parser_tuple!(A~0, B~1);
impl_parser_tuple!(A~0, B~1, C~2);
impl_parser_tuple!(A~0, B~1, C~2, D~3);
impl_parser_tuple!(A~0, B~1, C~2, D~3, E~4);
impl_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5);
impl_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6);
impl_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7);
impl_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8);
impl_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9);
impl_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10);
impl_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10, L~11);

/// Helper trait for interacting with tuples of parsers
pub trait SameParserTuple<Input>: ParserTuple<Input> {
    type Output: Debug + Clone + 'static;

    /// Call Parser::annotate on a specific child parser
    fn annotate(
        &mut self,
        input: &mut Input,
        index: usize,
    ) -> Option<AnnotatedResult<Self::Output>>;

    /// Call Parser::parse on a specific child parser
    fn parse(&mut self, input: &mut Input, index: usize) -> Option<ParseResult<Self::Output>>;
}

macro_rules! impl_same_parser_tuple {
    ( $First:ident ~ $first_idx:tt $(, $P:ident ~ $idx:tt )* ) => {
        paste! {
            impl<Input, $First $(, $P)*> SameParserTuple<Input> for ($First, $($P,)*)
            where
                $First: Parser<Input>,
                $(
                    $P: Parser<Input, Output = $First::Output>,
                )*
            {
                type Output = $First::Output;

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

impl_same_parser_tuple!(A~0);
impl_same_parser_tuple!(A~0, B~1);
impl_same_parser_tuple!(A~0, B~1, C~2);
impl_same_parser_tuple!(A~0, B~1, C~2, D~3);
impl_same_parser_tuple!(A~0, B~1, C~2, D~3, E~4);
impl_same_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5);
impl_same_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6);
impl_same_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7);
impl_same_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8);
impl_same_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9);
impl_same_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10);
impl_same_parser_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9, K~10, L~11);
