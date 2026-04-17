use std::cmp::Ordering;

use crate::{Annotation, Parser, ParserSpec};

use super::TakeArray;

impl<const N: usize> Parser<&str> for TakeArray<N> {
    type Output = String;

    #[inline(always)]
    fn name(&self) -> String {
        format!("take({})", N)
    }

    #[inline(always)]
    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(Parser::<&str>::name(self))
    }

    #[inline(always)]
    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: crate::AnnotationMode,
    ) -> crate::parser::ParseWithResult<Self::Output> {
        let end = match input.chars().count().cmp(&N) {
            Ordering::Less => {
                let annotation = if annotation_mode.fail {
                    Annotation::incomplete(Parser::<&str>::name(self), 0, vec![]).into()
                } else {
                    0.into()
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
            (0..N).into()
        };

        Ok((value, annotation))
    }
}
