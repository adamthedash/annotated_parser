mod parser;
mod value;

pub use parser::Store;
pub use value::{ForwardRef, ForwardRefGet, ForwardRefTuple, ForwrdRefSet};

use crate::Parser;

/// For the Store combinator and passthroughs
pub trait StoringParser<Input>: Parser<Input> {
    /// Type of inner value
    type Value;
    /// Type of forward reference
    type Ref;

    /// Obtain a handle to the output of this parser. May or may not be initialised yet.
    fn output(&self) -> Self::Ref;
}
