use crate::{AnnotatedResult, Annotation, ByteParser, ParseResult, Parser, ParserSpec};

#[derive(Clone)]
pub struct F16LE;

impl<'a> Parser<'a> for F16LE {
    type Input = &'a [u8];

    type Output = f16;

    fn name(&self) -> String {
        "le_f16".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn annotate(&mut self, input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        let value = f16::from_le_bytes(*bytes);

        // Move input along
        *input = rest;

        const BYTE_SIZE: usize = std::mem::size_of::<f16>();
        let annotation = Annotation::success(&self.name(), 0..BYTE_SIZE, value, vec![]);

        Ok((value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut Self::Input) -> crate::ParseResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        let value = f16::from_le_bytes(*bytes);

        // Move input along
        *input = rest;

        const BYTE_SIZE: usize = std::mem::size_of::<f16>();

        Ok((value, BYTE_SIZE))
    }
}

#[derive(Clone)]
pub struct F16BE;

impl<'a> Parser<'a> for F16BE {
    type Input = &'a [u8];

    type Output = f16;

    fn name(&self) -> String {
        "be_f16".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn annotate(&mut self, input: &mut Self::Input) -> AnnotatedResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        let value = f16::from_be_bytes(*bytes);

        // Move input along
        *input = rest;

        const BYTE_SIZE: usize = std::mem::size_of::<f16>();
        let annotation = Annotation::success(&self.name(), 0..BYTE_SIZE, value, vec![]);

        Ok((value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut Self::Input) -> crate::ParseResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        let value = f16::from_be_bytes(*bytes);

        // Move input along
        *input = rest;

        const BYTE_SIZE: usize = std::mem::size_of::<f16>();

        Ok((value, BYTE_SIZE))
    }
}
