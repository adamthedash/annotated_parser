use std::{fmt::Debug, marker::PhantomData};

use num_traits::FromBytes;

use crate::{Annotation, Parser, ParserSpec, Result};

/// Little-endian parser for types which can be directly interpreted from a byte array
#[derive(Clone)]
pub struct LE<T>(PhantomData<T>);

impl<const N: usize, T> Parser for LE<T>
where
    T: FromBytes<Bytes = [u8; N]>,
    T: Debug,
{
    type Output = T;

    fn name(&self) -> String {
        format!("le_{}", std::any::type_name::<T>())
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        let value = T::from_le_bytes(bytes);

        // Move input along
        *input = rest;

        let annotation = Annotation::success(&self.name(), 0..N, &value, vec![]);

        Ok((value, annotation))
    }
}

/// Big-endian parser for types which can be directly interpreted from a byte array
#[derive(Clone)]
pub struct BE<T>(PhantomData<T>);

impl<const N: usize, T> Parser for BE<T>
where
    T: FromBytes<Bytes = [u8; N]>,
    T: Debug,
{
    type Output = T;

    fn name(&self) -> String {
        format!("be_{}", std::any::type_name::<T>())
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let Some((bytes, rest)) = input.split_first_chunk() else {
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        };

        let value = T::from_be_bytes(bytes);

        // Move input along
        *input = rest;

        let annotation = Annotation::success(&self.name(), 0..1, &value, vec![]);

        Ok((value, annotation))
    }
}

pub trait ByteParser: Sized {
    const LE: LE<Self>;
    const BE: BE<Self>;
}

impl<const N: usize, T> ByteParser for T
where
    T: FromBytes<Bytes = [u8; N]>,
    T: Debug,
{
    const LE: LE<Self> = LE(PhantomData);
    const BE: BE<Self> = BE(PhantomData);
}
