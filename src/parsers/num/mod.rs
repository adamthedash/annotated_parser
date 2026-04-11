use crate::{Annotation, Parser, ParserSpec, Result};

mod endian;
#[cfg(feature = "f16")]
mod nightly_floats;
#[cfg(feature = "f16")]
pub use nightly_floats::F16LE;

pub use endian::{BE, ByteParser, LE};

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

    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
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

        Ok((value, 1))
    }
}
