use num_traits::AsPrimitive;

use crate::{
    Annotation, FoldResult, Parser, ParserSpec, Result, SpeedyResult, helpers::fold_child_err,
};

pub struct LengthRepeat<L, V> {
    length: L,
    value: V,
}

impl<L, V> LengthRepeat<L, V> {
    pub fn new(length_parser: L, value_parser: V) -> Self {
        Self {
            length: length_parser,
            value: value_parser,
        }
    }
}

impl<L, V> Parser for LengthRepeat<L, V>
where
    L: Parser,
    L::Output: AsPrimitive<usize>,
    V: Parser,
{
    type Output = Vec<V::Output>;

    #[inline(always)]
    fn name(&self) -> String {
        "length_repeat".to_owned()
    }

    #[inline(always)]
    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.length.spec(), self.value.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let name = self.name();

        let (length, mut offset, mut child_annotations) =
            self.length.parse(input).fold(vec![], 0, &name, 0)?;
        let length = length.as_();
        child_annotations.reserve(length);

        let mut values = Vec::with_capacity(length);
        for _ in 0..length {
            let value;
            (value, offset, child_annotations) =
                self.value
                    .parse(input)
                    .fold(child_annotations, offset, &name, 1)?;

            values.push(value);
        }

        let annotation = Annotation::success(name, 0..offset, values.clone(), child_annotations);

        Ok((values, annotation))
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> SpeedyResult<Self::Output> {
        let (length, mut offset) = self
            .length
            .parse_speedy(input)
            .map_err(|a| fold_child_err(a, vec![], 0, &self.name(), 0))?;
        let length = length.as_();

        let mut values = Vec::with_capacity(length);
        for _ in 0..length {
            let value;
            (value, offset) = self
                .value
                .parse_speedy(input)
                .map_err(|a| fold_child_err(a, vec![], offset, &self.name(), 1))?;

            values.push(value);
        }

        Ok((values, offset))
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
        let (value, anno) = parser.parse(input).unwrap();
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
    fn test_length_repeat_bad() {
        let bytes = [2, 0, 0, 0, 1, 0];
        let input = &mut bytes.as_slice();

        let mut parser = LengthRepeat::new(u32::LE, u16::LE);
        let anno = parser.parse(input).unwrap_err();
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
