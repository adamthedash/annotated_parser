use crate::{Annotation, Parser, ParserSpec, Result};

#[derive(Clone)]
pub struct U32LE;

impl Parser for U32LE {
    type Output = u32;

    fn name(&self) -> String {
        "le_u32".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        let value = u32::from_le_bytes(*bytes);

        // Move input along
        *input = rest;

        const BYTE_SIZE: usize = std::mem::size_of::<u32>();
        let annotation = Annotation::success(&self.name(), 0..BYTE_SIZE, value, vec![]);

        Ok((value, annotation))
    }
}

#[derive(Clone)]
pub struct U16LE;

impl Parser for U16LE {
    type Output = u16;

    fn name(&self) -> String {
        "le_u16".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        let value = u16::from_le_bytes(*bytes);

        // Move input along
        *input = rest;

        const BYTE_SIZE: usize = std::mem::size_of::<u16>();
        let annotation = Annotation::success(&self.name(), 0..BYTE_SIZE, value, vec![]);

        Ok((value, annotation))
    }
}

#[derive(Clone)]
pub struct U8;

impl Parser for U8 {
    type Output = u8;

    fn name(&self) -> String {
        "u8".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let Some((value, rest)) = input.split_first() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        // Move input along
        *input = rest;

        let annotation = Annotation::success(&self.name(), 0..1, value, vec![]);

        Ok((*value, annotation))
    }
}

/// 0 or 1 stored in u8
#[derive(Clone)]
pub struct Bool;

impl Parser for Bool {
    type Output = bool;

    fn name(&self) -> String {
        "bool".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let Some((first, rest)) = input.split_first() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        let value = match first {
            0 => false,
            1 => true,
            x => {
                return Err(Annotation::invalid(
                    &self.name(),
                    0..1,
                    format!("Invalid bool value: {x}"),
                    vec![],
                ));
            }
        };

        // Move input along
        *input = rest;

        let annotation = Annotation::success(&self.name(), 0..1, value, vec![]);

        Ok((value, annotation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AnnotationResult;

    #[test]
    fn test_u32_good() {
        let bytes = [4, 0, 0, 0];
        let input = &mut bytes.as_slice();

        let (value, anno) = U32LE.parse(input).unwrap();
        assert_eq!(value, 4);
        assert_eq!(anno.parser_id, "le_u32");
        assert!(anno.children.is_empty());

        let AnnotationResult::Success { span, value } = anno.result else {
            unreachable!()
        };

        assert_eq!(span, 0..4);
        assert_eq!(value, "4");
    }

    #[test]
    fn test_u32_bad() {
        let bytes = [4, 0, 0];
        let input = &mut bytes.as_slice();

        let anno = U32LE.parse(input).unwrap_err();
        assert_eq!(anno.parser_id, "le_u32");
        assert!(anno.children.is_empty());

        let AnnotationResult::Incomplete { start } = anno.result else {
            unreachable!()
        };

        assert_eq!(start, 0);
    }
}
