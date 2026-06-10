mod endian;
pub use endian::{BE, ByteParser, LE};

#[cfg(feature = "f16")]
mod nightly_floats;
#[cfg(feature = "f16")]
pub use nightly_floats::{F16BE, F16LE};

use crate::{Annotation, AnnotationReturn, Parser, ParserSpec};

/// Parse a boolean value from a single byte.
///
/// Expects `0` for `false` and `1` for `true`. Any other byte value is an error.
/// Fails if the input is empty.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::byte::Bool;
///
/// let mut input = &[1_u8][..];
/// let (value, _) = Bool.parse(&mut input).unwrap();
/// assert_eq!(value, true);
/// ```
#[derive(Clone)]
pub struct Bool;

impl Parser<&[u8]> for Bool {
    type Output = bool;

    fn name(&self) -> String {
        "bool".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: crate::AnnotationMode,
    ) -> crate::parser::ParseWithResult<Self::Output> {
        let Some((first, rest)) = input.split_first() else {
            let annotation = if annotation_mode.fail {
                Annotation::incomplete(self.name(), 0, vec![]).into()
            } else {
                AnnotationReturn::Start(0)
            };
            return Err(annotation);
        };

        let value = match first {
            0 => false,
            1 => true,
            x => {
                let annotation = if annotation_mode.fail {
                    Annotation::invalid(
                        self.name(),
                        0..1,
                        format!("Invalid bool value: {x}"),
                        vec![],
                    )
                    .into()
                } else {
                    AnnotationReturn::Span(0..1)
                };
                return Err(annotation);
            }
        };

        // Move input along
        *input = rest;

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..1, value, vec![]).into()
        } else {
            AnnotationReturn::Span(0..1)
        };

        Ok((value, annotation))
    }
}
