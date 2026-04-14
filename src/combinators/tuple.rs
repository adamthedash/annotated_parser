use crate::{
    AnnotatedResult, Annotation, FoldResult, ParseResult, Parser, ParserSpec,
    helpers::fold_child_err,
};
use paste::paste;

/// Tuples of parsers
macro_rules! impl_parser_for_tuple {
    ( $First:ident ~ $first_idx:tt $(, $P:ident ~ $idx:tt )* ) => {
        paste! {
            impl<'a, $First $(, $P)*> Parser<'a> for ($First, $($P,)*)
            where
                $First: Parser<'a>,
                $(
                    $P: Parser<'a, Input = $First::Input>,
                )*
            {
                type Input = $First::Input;
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

                fn annotate(&mut self, input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
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
                fn parse(&mut self, input: &mut Self::Input) -> ParseResult<Self::Output> {
                    let mut offset = 0usize;

                    let [<out_ $first_idx>];
                    ([<out_ $first_idx>], offset) =
                        self.$first_idx
                            .parse(input)
                            .map_err(|a| fold_child_err(a, vec![], offset, &self.name(), $first_idx))?;

                    $(
                        let [<out_ $idx>];
                        ([<out_ $idx>], offset) =
                            self.$idx
                                .parse(input)
                                .map_err(|a| fold_child_err(a, vec![], offset, &self.name(), $idx))?;
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
