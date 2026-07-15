use crate::{
    Annotation, AnnotationMode, AnnotationReturn, FoldParseWithResult, Parser, ParserSpec,
    combinators::{Checkpoint, Peek},
};

/// Apply a parser repeatedly until a terminator succeeds, without consuming the terminator.
///
/// Runs the inner parser zero or more times, stopping as soon as the terminator parser
/// succeeds. The terminator is not consumed from the input.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
///
/// let mut parser = "hello".repeat_till_exc("world");
/// let mut input = "hellohelloworld";
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value, vec!["hello", "hello"]);
/// assert_eq!(input, "world");
/// ```
pub struct RepeatTillExc<P, T> {
    inner: P,
    terminator: Peek<T>,
}

impl<P, T> RepeatTillExc<P, T> {
    pub fn new<Input>(inner: P, terminator: T) -> Self
    where
        P: Parser<Input>,
        T: Parser<Input>,
        Input: Copy,
    {
        Self {
            inner,
            terminator: Peek::new(terminator),
        }
    }
}

impl<Input, P, T> Parser<Input> for RepeatTillExc<P, T>
where
    P: Parser<Input>,
    T: Parser<Input>,
    Input: Copy,
{
    type Output = Vec<P::Output>;

    fn name(&self) -> String {
        "repeat_till_exc".to_owned()
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec(), self.terminator.spec()])
    }

    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> crate::ParseWithResult<Self::Output> {
        let mut child_annotations = annotation_mode.success.then(Vec::new);
        let mut values = vec![];
        let mut offset = 0;

        while self
            .terminator
            .parse_with(input, AnnotationMode::NONE)
            .is_err()
        {
            let val;
            (val, offset, child_annotations) = self.inner.parse_with(input, annotation_mode).fold(
                annotation_mode,
                || self.name(),
                child_annotations,
                offset,
                0,
            )?;
            values.push(val);
        }

        let annotation = if let Some(child_annotations) = child_annotations {
            Annotation::success(self.name(), 0..offset, values.clone(), child_annotations).into()
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok((values, annotation))
    }
}

/// Apply a parser repeatedly until a terminator succeeds, including the terminator in the result.
///
/// Runs the inner parser zero or more times, stopping when the terminator parser succeeds.
/// The terminator is consumed and its value is returned alongside the collected inner parses.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
///
/// let mut parser = "hello".repeat_till_inc("world");
/// let mut input = "hellohelloworld";
/// let ((values, term), _) = parser.parse(&mut input).unwrap();
/// assert_eq!(values, vec!["hello", "hello"]);
/// assert_eq!(term, "world");
/// assert_eq!(input, "");
/// ```
pub struct RepeatTillInc<P, T> {
    inner: P,
    terminator: Checkpoint<T>,
}

impl<P, T> RepeatTillInc<P, T> {
    pub fn new<Input>(inner: P, terminator: T) -> Self
    where
        P: Parser<Input>,
        T: Parser<Input>,
        Input: Copy,
    {
        Self {
            inner,
            terminator: Checkpoint::new(terminator),
        }
    }
}

impl<Input, P, T> Parser<Input> for RepeatTillInc<P, T>
where
    P: Parser<Input>,
    T: Parser<Input>,
    Input: Copy,
{
    type Output = (Vec<P::Output>, T::Output);

    fn name(&self) -> String {
        "repeat_till_inc".to_owned()
    }

    fn spec(&self) -> crate::ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec(), self.terminator.spec()])
    }

    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: crate::AnnotationMode,
    ) -> crate::ParseWithResult<Self::Output> {
        let mut child_annotations = annotation_mode.success.then(Vec::new);
        let mut values = vec![];
        let mut offset = 0;

        let term_anno_mode = AnnotationMode {
            fail: false,
            ..annotation_mode
        };

        let term_value;
        loop {
            let res = self.terminator.parse_with(input, term_anno_mode);
            if res.is_ok() {
                // Terminator found
                (term_value, offset, child_annotations) = res
                    .fold(
                        annotation_mode,
                        || self.name(),
                        child_annotations,
                        offset,
                        1,
                    )
                    .expect("Happy path");

                break;
            }

            let val;
            (val, offset, child_annotations) = self.inner.parse_with(input, annotation_mode).fold(
                annotation_mode,
                || self.name(),
                child_annotations,
                offset,
                0,
            )?;
            values.push(val);
        }

        let annotation = if let Some(child_annotations) = child_annotations {
            Annotation::success(
                self.name(),
                0..offset,
                (values.clone(), term_value.clone()),
                child_annotations,
            )
            .into()
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok(((values, term_value), annotation))
    }
}

#[cfg(test)]
mod tests {
    use crate::AnnotationResult;

    use super::*;

    mod exc {
        use super::*;

        #[test]
        fn test_empty() {
            let mut input = "world";
            let mut parser = RepeatTillExc::new("hello", "world");

            let (value, offset) = parser.parse(&mut input).unwrap();
            assert!(value.is_empty());
            assert_eq!(offset, 0);
            assert_eq!(input, "world");
        }

        #[test]
        fn test_good() {
            let mut input = "hellohelloworld";
            let mut parser = RepeatTillExc::new("hello", "world");

            let (value, offset) = parser.parse(&mut input).unwrap();
            assert_eq!(value, ["hello", "hello"]);
            assert_eq!(offset, 10);
            assert_eq!(input, "world");
        }

        #[test]
        fn test_bad_inner() {
            let mut input = "hellohellyworld";
            let mut parser = RepeatTillExc::new("hello", "world");

            let anno = parser.parse(&mut input).unwrap_err();
            assert!(
                matches!(anno.result, AnnotationResult::Child { .. }),
                "{:?}",
                anno
            );
            assert!(
                matches!(anno.children[0].result, AnnotationResult::Invalid { .. }),
                "{:?}",
                anno
            );
        }

        #[test]
        fn test_bad_terminator() {
            let mut input = "hellohelloworly";
            let mut parser = RepeatTillExc::new("hello", "world");

            let anno = parser.parse(&mut input).unwrap_err();
            assert!(
                matches!(anno.result, AnnotationResult::Child { .. }),
                "{:?}",
                anno
            );
            assert!(
                matches!(anno.children[0].result, AnnotationResult::Invalid { .. }),
                "{:?}",
                anno
            );
        }
    }

    mod inc {
        use super::*;

        #[test]
        fn test_empty() {
            let mut input = "world";
            let mut parser = RepeatTillInc::new("hello", "world");

            let ((value, term), offset) = parser.parse(&mut input).unwrap();
            assert!(value.is_empty());
            assert_eq!(term, "world");
            assert_eq!(offset, 5);
            assert_eq!(input, "");
        }

        #[test]
        fn test_good() {
            let mut input = "hellohelloworld";
            let mut parser = RepeatTillInc::new("hello", "world");

            let ((value, term), offset) = parser.parse(&mut input).unwrap();
            assert_eq!(value, ["hello", "hello"]);
            assert_eq!(term, "world");
            assert_eq!(offset, 15);
            assert_eq!(input, "");
        }

        #[test]
        fn test_bad_inner() {
            let mut input = "hellohellyworld";
            let mut parser = RepeatTillInc::new("hello", "world");

            let anno = parser.parse(&mut input).unwrap_err();
            assert!(
                matches!(anno.result, AnnotationResult::Child { .. }),
                "{:?}",
                anno
            );
            assert!(
                matches!(anno.children[0].result, AnnotationResult::Invalid { .. }),
                "{:?}",
                anno
            );
        }

        #[test]
        fn test_bad_terminator() {
            let mut input = "hellohelloworly";
            let mut parser = RepeatTillInc::new("hello", "world");

            let anno = parser.parse(&mut input).unwrap_err();
            assert!(
                matches!(anno.result, AnnotationResult::Child { .. }),
                "{:?}",
                anno
            );
            assert!(
                matches!(anno.children[0].result, AnnotationResult::Invalid { .. }),
                "{:?}",
                anno
            );
        }
    }
}
