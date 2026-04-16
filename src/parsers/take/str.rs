use std::cmp::Ordering;

use crate::{AnnotatedResult, Annotation, Parser, ParserSpec};

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

    fn annotate(&mut self, input: &mut &str) -> AnnotatedResult<Self::Output> {
        let end = match input.chars().count().cmp(&N) {
            Ordering::Less => {
                return Err(Annotation::incomplete(
                    Parser::<&str>::name(self),
                    0,
                    vec![],
                ));
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

        let annotation =
            Annotation::success(Parser::<&str>::name(self), 0..N, value.clone(), vec![]);

        Ok((value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut &str) -> crate::ParseResult<Self::Output> {
        let end = match input.chars().count().cmp(&N) {
            Ordering::Less => {
                return Err(Annotation::incomplete(
                    Parser::<&str>::name(self),
                    0,
                    vec![],
                ));
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

        Ok((value, N))
    }
}
