mod parser;
mod value;

pub use parser::Store;
pub use value::{ForwardRef, ForwardRefGet, ForwardRefTuple, ForwrdRefSet};

use crate::Parser;

/// A parser that stores its output in a `ForwardRef`.
///
/// This trait is implemented by the [`Store`] combinator. It provides a way to
/// access the stored value handle after the parser is constructed, so that
/// other parsers can reference it before the parse runs.
pub trait StoringParser<Input>: Parser<Input> {
    /// Type of the inner parsed value.
    type Value;
    /// Type of the forward reference handle.
    type Ref;

    /// Obtain a handle to the output of this parser. May or may not be initialised yet.
    fn output(&self) -> Self::Ref;
}
