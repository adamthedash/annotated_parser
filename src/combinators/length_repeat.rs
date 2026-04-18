use num_traits::AsPrimitive;

use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec, helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

pub struct LengthRepeat<L, V> {
    length: L,
    value: V,
}

impl<L, V> LengthRepeat<L, V> {
    pub fn new<Input>(length_parser: L, value_parser: V) -> Self
    where
        L: Parser<Input>,
        L::Output: AsPrimitive<usize>,
        V: Parser<Input>,
    {
        Self {
            length: length_parser,
            value: value_parser,
        }
    }
}

impl<Input, L, V> Parser<Input> for LengthRepeat<L, V>
where
    L: Parser<Input>,
    L::Output: AsPrimitive<usize>,
    V: Parser<Input>,
{
    type Output = Vec<V::Output>;

    #[inline]
    fn name(&self) -> String {
        "length_repeat".to_owned()
    }

    #[inline]
    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.length.spec(), self.value.spec()])
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let mut child_annotations = annotation_mode.success.then(Vec::new);
        let mut offset = 0;

        // Length
        let length;
        (length, offset, child_annotations) = self.length.parse_with(input, annotation_mode).fold(
            annotation_mode,
            || self.name(),
            child_annotations,
            offset,
            0,
        )?;
        let length = length.as_();
        if let Some(a) = &mut child_annotations {
            a.reserve_exact(length);
        }

        // Repeat
        let mut values = Vec::with_capacity(length);
        let mut value;
        for _ in 0..length {
            (value, offset, child_annotations) =
                self.value.parse_with(input, annotation_mode).fold(
                    annotation_mode,
                    || self.name(),
                    child_annotations,
                    offset,
                    1,
                )?;

            values.push(value);
        }

        let annotation = if annotation_mode.success {
            Annotation::success(
                self.name(),
                0..offset,
                values.clone(),
                child_annotations.unwrap(),
            )
            .into()
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok((values, annotation))
    }
}

#[cfg(test)]
mod tests {
    use crate::AnnotationResult;
    use crate::ByteParser;

    use super::*;

    #[test]
    fn test_length_repeat_good() {
        let bytes = [2, 0, 0, 0, 1, 0, 2, 0];
        let input = &mut bytes.as_slice();

        let mut parser = LengthRepeat::new(u32::LE, u16::LE);
        let (value, anno) = parser.annotate(input).unwrap();
        assert_eq!(value, vec![1, 2]);
        assert_eq!(anno.parser_id, "length_repeat");
        assert_eq!(anno.children.len(), 3);

        let AnnotationResult::Success { span, value } = &anno.result else {
            unreachable!()
        };

        assert_eq!(*span, 0..8);
        assert_eq!(format!("{value:?}"), "[1, 2]");
    }

    #[test]
    fn test_length_repeat_parse() {
        let bytes = [2, 0, 0, 0, 1, 0, 2, 0];
        let input = &mut bytes.as_slice();

        let mut parser = LengthRepeat::new(u32::LE, u16::LE);
        let (value, offset) = parser.parse(input).unwrap();
        assert_eq!(value, vec![1, 2]);

        assert_eq!(offset, 8);
        assert_eq!(format!("{value:?}"), "[1, 2]");
    }

    #[test]
    fn test_length_repeat_bad() {
        let bytes = [2, 0, 0, 0, 1, 0];
        let input = &mut bytes.as_slice();

        let mut parser = LengthRepeat::new(u32::LE, u16::LE);
        let anno = parser.annotate(input).unwrap_err();
        assert_eq!(anno.parser_id, "length_repeat");
        assert_eq!(anno.children.len(), 3);
    }

    #[test]
    fn test_length_repeat_spec() {
        let parser = LengthRepeat::new(u32::LE, u16::LE);
        let spec = parser.spec();

        let expected = ParserSpec {
            name: "length_repeat".to_owned(),
            inner: vec![ParserSpec::empty("le_u32"), ParserSpec::empty("le_u16")],
            friendly_name: None,
        };

        assert_eq!(expected, spec);
    }
}
