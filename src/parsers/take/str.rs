use std::cmp::Ordering;

use crate::{
    Annotation, AnnotationMode, AnnotationReturn, ParserExec, ParserInfo, parser::ParseWithResult,
};

use super::TakeArray;

impl<const N: usize> ParserExec<&str> for TakeArray<N> {
    type Output = String;

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let end = match input.chars().count().cmp(&N) {
            Ordering::Less => {
                let annotation = if annotation_mode.fail {
                    Annotation::incomplete(self.name(), 0, vec![]).into()
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
            Annotation::success(self.name(), 0..N, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok((value, annotation))
    }
}
