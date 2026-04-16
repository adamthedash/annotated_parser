use crate::{Annotation, Parser, ParserSpec};

impl<const N: usize> Parser<&[u8]> for &'static [u8; N] {
    type Output = &'static [u8; N];

    fn name(&self) -> String {
        format!("literal({:?})", self)
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn annotate(&mut self, input: &mut &[u8]) -> crate::AnnotatedResult<Self::Output> {
        if !input.starts_with(*self) {
            let annotation = if input.len() < N {
                Annotation::incomplete(self.name(), 0, vec![])
            } else {
                Annotation::invalid(
                    self.name(),
                    0..N,
                    format!("Expected {self:?}, found {:?}", &input[..self.len()]),
                    vec![],
                )
            };

            return Err(annotation);
        }

        *input = &input[N..];

        let annotation = Annotation::success(self.name(), 0..N, *self, vec![]);

        Ok((*self, annotation))
    }

    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        if !input.starts_with(*self) {
            let annotation = if input.len() < self.len() {
                Annotation::incomplete(self.name(), 0, vec![])
            } else {
                Annotation::invalid(
                    self.name(),
                    0..self.len(),
                    format!("Expected {self:?}, found {:?}", &input[..self.len()]),
                    vec![],
                )
            };

            return Err(annotation);
        }

        *input = &input[self.len()..];

        Ok((*self, self.len()))
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

    fn annotate(&mut self, input: &mut &str) -> crate::AnnotatedResult<Self::Output> {
        if !input.starts_with(*self) {
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

            return Err(annotation);
        }

        *input = &input[self.len()..];

        let num_chars = self.chars().count();

        let annotation = Annotation::success(self.name(), 0..num_chars, *self, vec![]);

        Ok((*self, annotation))
    }

    fn parse(&mut self, input: &mut &str) -> crate::ParseResult<Self::Output> {
        if !input.starts_with(*self) {
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

            return Err(annotation);
        }

        *input = &input[self.len()..];

        let num_chars = self.chars().count();

        Ok((*self, num_chars))
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
