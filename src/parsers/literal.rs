use crate::{
    Annotation, Parser, ParserSpec,
    parser::{AnnotationMode, AnnotationReturn, ParseWithResult},
};

impl<const N: usize> Parser<&[u8]> for &'static [u8; N] {
    type Output = &'static [u8; N];

    fn name(&self) -> String {
        format!("literal({:x?})", self)
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::empty(self.name())
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        if !input.starts_with(*self) {
            let annotation = if annotation_mode.fail {
                let annotation = if input.len() < N {
                    Annotation::incomplete(self.name(), 0, vec![])
                } else {
                    Annotation::invalid(
                        self.name(),
                        0..N,
                        format!("Expected {self:x?}, found {:x?}", &input[..N]),
                        vec![],
                    )
                };

                annotation.into()
            } else {
                if input.len() < N {
                    AnnotationReturn::Start(0)
                } else {
                    AnnotationReturn::Span(0..N)
                }
            };

            return Err(annotation);
        }

        *input = &input[N..];

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..N, *self, vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok((*self, annotation))
    }
}

// NOTE: Unfortunately this conflicts with str::parse, so it must be called with Parser::parse if
// using on its own
impl Parser<&str> for &'static str {
    type Output = &'static str;

    fn name(&self) -> String {
        format!("literal({:?})", self)
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::empty(self.name())
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &str,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        if !input.starts_with(*self) {
            let annotation = if annotation_mode.fail {
                let annotation = if input.len() < self.len() {
                    Annotation::incomplete(self.name(), 0, vec![])
                } else {
                    let num_chars = self.chars().count();
                    Annotation::invalid(
                        self.name(),
                        0..num_chars,
                        format!("Expected {self:?}, found {:?}", &input[..self.len()]),
                        vec![],
                    )
                };

                annotation.into()
            } else {
                if input.len() < self.len() {
                    AnnotationReturn::Start(0)
                } else {
                    let num_chars = self.chars().count();
                    AnnotationReturn::Span(0..num_chars)
                }
            };

            return Err(annotation);
        }

        *input = &input[self.len()..];

        let num_chars = self.chars().count();

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..num_chars, *self, vec![]).into()
        } else {
            AnnotationReturn::Span(0..num_chars)
        };

        Ok((*self, annotation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod str {
        use super::*;

        #[test]
        fn test_good() {
            let mut parser = "hello";

            let mut input = "hello_world";
            let (value, _) = Parser::parse(&mut parser, &mut input).unwrap();
            assert_eq!(value, "hello");
            assert_eq!(input, "_world");
        }

        #[test]
        fn test_bad() {
            let mut parser = "henlo";

            let mut input = "hello_world";
            let annotation = Parser::parse(&mut parser, &mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Invalid { .. }
            ));
            assert_eq!(input, "hello_world");
        }

        #[test]
        fn test_short() {
            let mut parser = "hello";

            let mut input = "hel";
            let annotation = Parser::parse(&mut parser, &mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Incomplete { .. }
            ));
            assert_eq!(input, "hel");
        }
    }

    mod byte {
        use super::*;

        #[test]
        fn test_good() {
            let mut parser = b"hello";

            let mut input = b"hello_world".as_slice();
            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, b"hello");
            assert_eq!(input[..], b"_world"[..]);
        }

        #[test]
        fn test_bad() {
            let mut parser = b"henlo";

            let mut input = b"hello_world".as_slice();
            let annotation = parser.parse(&mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Invalid { .. }
            ));
            assert_eq!(input, b"hello_world");
        }

        #[test]
        fn test_short() {
            let mut parser = b"hello";

            let mut input = b"hel".as_slice();
            let annotation = parser.parse(&mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Incomplete { .. }
            ));
            assert_eq!(input, b"hel");
        }
    }
}
