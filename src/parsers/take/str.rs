use std::cmp::Ordering;

use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec, parser::ParseWithResult,
};

use super::TakeArray;

impl<const N: usize> Parser<&str> for TakeArray<N> {
    type Output = String;

    #[inline]
    fn name(&self) -> String {
        format!("take({})", N)
    }

    #[inline]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<&str>::name(self))
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let end = match input.chars().count().cmp(&N) {
            Ordering::Less => {
                let annotation = if annotation_mode.fail {
                    Annotation::incomplete(Parser::<&str>::name(self), 0, vec![]).into()
                } else {
                    AnnotationReturn::Start(0)
                };
                return Err(annotation);
            }
            Ordering::Equal => input.len(),
            Ordering::Greater => {
                let (end, _) = input
                    .char_indices()
                    .nth(N)
                    .expect("length verified by match predicate");
                end
            }
        };

        let value = input[..end].to_string();

        *input = &input[end..];

        let annotation = if annotation_mode.success {
            Annotation::success(Parser::<&str>::name(self), 0..N, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok((value, annotation))
    }
}
