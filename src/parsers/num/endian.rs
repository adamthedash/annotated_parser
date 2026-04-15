use std::{fmt::Debug, marker::PhantomData};

use num_traits::FromBytes;

use crate::{AnnotatedResult, Annotation, Parser, ParserSpec};

/// Little-endian parser for types which can be directly interpreted from a byte array
#[derive(Clone)]
pub struct LE<T>(PhantomData<T>);

impl<const N: usize, T> Parser<&[u8]> for LE<T>
where
    T: FromBytes<Bytes = [u8; N]>,
    T: Debug + Clone + 'static,
{
    type Output = T;

    #[inline(always)]
    fn name(&self) -> String {
        // Concat is slightly faster than format!
        ["le_", std::any::type_name::<T>()].concat()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(self.name(), 0, vec![]));
        };

        let value = T::from_le_bytes(bytes);

        // Move input along
        *input = rest;

        let annotation = Annotation::success(self.name(), 0..N, value.clone(), vec![]);

        Ok((value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(self.name(), 0, vec![]));
        };

        let value = T::from_le_bytes(bytes);

        // Move input along
        *input = rest;

        Ok((value, N))
    }
}

/// Big-endian parser for types which can be directly interpreted from a byte array
#[derive(Clone)]
pub struct BE<T>(PhantomData<T>);

impl<const N: usize, T> Parser<&[u8]> for BE<T>
where
    T: FromBytes<Bytes = [u8; N]>,
    T: Debug + Clone + 'static,
{
    type Output = T;

    #[inline(always)]
    fn name(&self) -> String {
        // Concat is slightly faster than format!
        ["be_", std::any::type_name::<T>()].concat()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(self.name(), 0, vec![]));
        };

        let value = T::from_be_bytes(bytes);

        // Move input along
        *input = rest;

        let annotation = Annotation::success(self.name(), 0..N, value.clone(), vec![]);

        Ok((value, annotation))
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(self.name(), 0, vec![]));
        };

        let value = T::from_be_bytes(bytes);

        // Move input along
        *input = rest;

        Ok((value, N))
    }
}

pub trait ByteParser: Sized {
    type LEParser;
    type BEParser;

    const LE: Self::LEParser;
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
