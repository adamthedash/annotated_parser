mod parser;
mod value;
mod value2;

pub use parser::Delayed;
// pub use value::{DelayedVal, DelayedValDerived};
pub use value2::{
    DelayedVal, DelayedValGet, DelayedValGetTuple, DelayedValSet, DelayedValSetTuple,
};

use crate::Parser;

/// For the Delayed combinator and passthroughs
pub trait DelayedParser<Input>: Parser<Input> {
    type Value;
    type DelayedValue;

    /// Obtain a handle to the output of this parser. May or may not be initialised yet.
    fn output(&self) -> Self::DelayedValue;
}
