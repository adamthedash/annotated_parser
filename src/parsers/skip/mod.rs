use num_traits::AsPrimitive;

use crate::ForwardRefGet;

mod byte;
mod str;

pub struct SkipArray<const N: usize>;

pub struct SkipVec<C>(C);

impl<C> SkipVec<C> {
    pub fn new(count: C) -> Self
    where
        C: ForwardRefGet,
        C::Value: AsPrimitive<usize>,
    {
        Self(count)
    }
}
