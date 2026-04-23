use crate::parser::ParseWithResult;
use crate::{Annotation, Parser, ParserSpec, helpers::FoldParseWithResult};
use crate::{AnnotationMode, AnnotationReturn};
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

                #[inline]
                fn parse_with(
                    &mut self,
                    input: &mut Input,
                    annotation_mode: crate::AnnotationMode,
                ) -> ParseWithResult<Self::Output> {
                    let mut child_annotations = annotation_mode.success.then(Vec::new);
                    let mut offset = 0usize;

                    let [<out_ $first_idx>];
                    ([<out_ $first_idx>], offset, child_annotations) =
                        self.$first_idx
                            .parse_with(input, annotation_mode)
                            .fold(annotation_mode, || self.name(), child_annotations, offset, $first_idx)?;

                    $(
                        let [<out_ $idx>];
                        ([<out_ $idx>], offset, child_annotations) =
                            self.$idx
                                .parse_with(input, annotation_mode)
                                .fold(annotation_mode, || self.name(), child_annotations, offset, $idx)?;
                    )*

                    let out = ([<out_ $first_idx>], $( [<out_ $idx>], )*);

                    let annotation = if annotation_mode.success {
                        Annotation::success(&self.name(), 0..offset, out.clone(), child_annotations.unwrap()).into()
                    } else {
                        AnnotationReturn::Span(0..offset)
                    };
                    Ok((out, annotation))
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

    /// Call Parser::parse_with on a specific child parser
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
        index: usize,
    ) -> Option<ParseWithResult<Self::Output>>;
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

                #[inline]
                fn parse_with(
                    &mut self,
                    input: &mut Input,
                    annotation_mode: AnnotationMode,
                    index: usize,
                ) -> Option<ParseWithResult<Self::Output>> {
                    match index {
                        $first_idx => Some(self.$first_idx.parse_with(input, annotation_mode)),
                        $( $idx => Some(self.$idx.parse_with(input, annotation_mode)), )*
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
