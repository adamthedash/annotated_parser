use crate::{Annotation, Parser, ParserSpec, Result};

#[derive(Clone)]
pub struct F16LE;

impl Parser for F16LE {
    type Output = f16;

    fn name(&self) -> String {
        "le_f16".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
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
}
