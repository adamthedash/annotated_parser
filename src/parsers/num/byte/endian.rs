use std::{fmt::Debug, marker::PhantomData};

use num_traits::FromBytes;

use crate::{
    Annotation, AnnotationReturn, Parser, ParserOutput, ParserSpec, parser::ParseWithResult,
};

/// Parse a value from its little-endian byte representation.
///
/// Consumes exactly `N` bytes from the input and interprets them as a little-endian
/// value of type `T`, where `N` is the byte size of `T`. Fails if the input is too short.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::byte::ByteParser;
///
/// let mut input = &[0x01, 0x00, 0x00, 0x00][..];
/// let (value, _) = u32::LE.parse(&mut input).unwrap();
/// assert_eq!(value, 1);
/// ```
#[derive(Clone)]
pub struct LE<T>(PhantomData<T>);

impl<const N: usize, T> Parser<&[u8]> for LE<T>
where
    T: FromBytes<Bytes = [u8; N]>,
    T: ParserOutput,
{
    type Output = T;

    #[inline]
    fn name(&self) -> String {
        // Concat is slightly faster than format!
        ["le_", std::any::type_name::<T>()].concat()
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

        let value = T::from_le_bytes(bytes);

        // Move input along
        *input = rest;

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..N, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok((value, annotation))
    }
}

/// Parse a value from its big-endian byte representation.
///
/// Consumes exactly `N` bytes from the input and interprets them as a big-endian
/// value of type `T`, where `N` is the byte size of `T`. Fails if the input is too short.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::byte::ByteParser;
///
/// let mut input = &[0x00, 0x00, 0x00, 0x01][..];
/// let (value, _) = u32::BE.parse(&mut input).unwrap();
/// assert_eq!(value, 1);
/// ```
#[derive(Clone)]
pub struct BE<T>(PhantomData<T>);

impl<const N: usize, T> Parser<&[u8]> for BE<T>
where
    T: FromBytes<Bytes = [u8; N]>,
    T: ParserOutput,
{
    type Output = T;

    #[inline]
    fn name(&self) -> String {
        // Concat is slightly faster than format!
        ["be_", std::any::type_name::<T>()].concat()
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

        let value = T::from_be_bytes(bytes);

        // Move input along
        *input = rest;

        let annotation = if annotation_mode.success {
            Annotation::success(self.name(), 0..N, value.clone(), vec![]).into()
        } else {
            AnnotationReturn::Span(0..N)
        };

        Ok((value, annotation))
    }
}

/// Provides little-endian and big-endian byte parsers for a type.
///
/// Implemented automatically for types that implement `FromBytes` (from `num_traits`).
/// Use the constants `LE` and `BE` to get the parsers.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::byte::ByteParser;
///
/// let mut input = &[0x01, 0x00, 0x00, 0x00][..];
/// let (value, _) = u32::LE.parse(&mut input).unwrap();
/// assert_eq!(value, 1);
/// ```
pub trait ByteParser: Sized {
    /// The little-endian parser for this type.
    type LEParser;
    /// The big-endian parser for this type.
    type BEParser;

    /// Little-endian parser instance.
    const LE: Self::LEParser;
    /// Big-endian parser instance.
    const BE: Self::BEParser;
}

impl<const N: usize, T> ByteParser for T
where
    T: FromBytes<Bytes = [u8; N]>,
    T: Debug,
{
    type LEParser = LE<Self>;
    type BEParser = BE<Self>;

    const LE: Self::LEParser = LE(PhantomData);
    const BE: Self::BEParser = BE(PhantomData);
}
