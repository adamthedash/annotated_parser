use crate::{AnnotationReturn, ParseWithResult, ParserExec, ParserInfo};

use crate::{Annotation, ParserSpec};

/// Consume all remaining input.
/// Always succeeds, even when the input is empty.
///
/// # Examples
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::Rest;
///
/// let mut input = "hello";
/// let (value, _) = Rest.parse(&mut input).unwrap();
/// assert_eq!(value, "hello");
/// assert_eq!(input, "");
/// ```
pub struct Rest;

impl ParserInfo for Rest {
    fn name(&self) -> String {
        "rest".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }
}

impl ParserExec<&[u8]> for Rest {
    type Output = Vec<u8>;

    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let value = input.to_owned();

        *input = &[];

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..value.len(), value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..value.len())
        };

        Ok((value, annotation))
    }
}

impl ParserExec<&str> for Rest {
    type Output = String;

    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let value = input.to_owned();
        let num_chars = value.chars().count();

        *input = "";

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..num_chars, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..num_chars)
        };

        Ok((value, annotation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod byte {
        use super::*;

        #[test]
        fn test_non_empty() {
            let mut input = [1, 2, 3].as_slice();
            let (value, _) = Rest.parse(&mut input).unwrap();
            assert_eq!(value, vec![1, 2, 3]);
            assert_eq!(input, []);
        }

        #[test]
        fn test_empty() {
            let mut input = [].as_slice();
            let (value, _) = Rest.parse(&mut input).unwrap();
            assert_eq!(value, Vec::<u8>::new());
            assert_eq!(input, []);
        }
    }

    mod str {
        use super::*;

        #[test]
        fn test_non_empty() {
            let mut input = "hello";
            let (value, _) = Rest.parse(&mut input).unwrap();
            assert_eq!(value, "hello");
            assert_eq!(input, "");
        }

        #[test]
        fn test_empty() {
            let mut input = "";
            let (value, _) = Rest.parse(&mut input).unwrap();
            assert_eq!(value, "");
            assert_eq!(input, "");
        }

        #[test]
        fn test_multibyte() {
            let mut input = "αβγ";
            let (value, _) = Rest.parse(&mut input).unwrap();
            assert_eq!(value, "αβγ");
            assert_eq!(input, "");
        }
    }
}
