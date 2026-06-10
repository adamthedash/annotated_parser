use crate::{Annotation, AnnotationReturn, ParseWithResult, Parser, ParserSpec};

/// Parse a half-precision float from its little-endian byte representation.
///
/// Consumes 2 bytes and interprets them as a little-endian `f16`.
/// Fails if the input is shorter than 2 bytes.
///
/// # Example
///
/// ```ignore
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::byte::F16LE;
///
/// let mut input = &[0x00, 0x3C][..];
/// let (value, _) = F16LE.parse(&mut input).unwrap();
/// ```
#[derive(Clone)]
pub struct F16LE;

impl Parser<&[u8]> for F16LE {
    type Output = f16;

    fn name(&self) -> String {
        "le_f16".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            let annotation = if annotation_mode.fail {
                Annotation::incomplete(self.name(), 0, vec![]).into()
            } else {
                AnnotationReturn::Start(0)
            };

            return Err(annotation);
        };

        let value = f16::from_le_bytes(*bytes);

        // Move input along
        *input = rest;

        const N: usize = std::mem::size_of::<f16>();
        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..N, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok((value, annotation))
    }
}

/// Parse a half-precision float from its big-endian byte representation.
///
/// Consumes 2 bytes and interprets them as a big-endian `f16`.
/// Fails if the input is shorter than 2 bytes.
///
/// # Example
///
/// ```ignore
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::byte::F16BE;
///
/// let mut input = &[0x3C, 0x00][..];
/// let (value, _) = F16BE.parse(&mut input).unwrap();
/// ```
#[derive(Clone)]
pub struct F16BE;

impl Parser<&[u8]> for F16BE {
    type Output = f16;

    fn name(&self) -> String {
        "be_f16".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    #[inline]
    fn parse_with(
        &mut self,
        input: &mut &[u8],
        annotation_mode: crate::AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            let annotation = if annotation_mode.fail {
                Annotation::incomplete(self.name(), 0, vec![]).into()
            } else {
                AnnotationReturn::Start(0)
            };

            return Err(annotation);
        };

        let value = f16::from_be_bytes(*bytes);

        // Move input along
        *input = rest;

        const N: usize = std::mem::size_of::<f16>();
        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..N, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok((value, annotation))
    }
}
