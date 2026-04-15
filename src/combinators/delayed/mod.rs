mod parser;
mod value;

use std::ops::Deref;

pub use parser::Delayed;
pub use value::{DelayedVal, DelayedValDerived};

use crate::Parser;

/// For Write/owner side
pub trait DelayedValSet {
    type Value;

    fn set(&self, value: Self::Value);
    fn take(&self) -> Self::Value;
}

/// For Read side
pub trait DelayedValGet {
    type Value;

    fn get(&self) -> impl Deref<Target = Self::Value>;

    /// Create a derived value by applying a function to this value
    /// NOTE: There's currently no way to specify "If the provided func is Clone, then the return
    /// is Clone". So just restrict ths to Clone func's for now.
    fn map<O>(
        self,
        func: impl Fn(&Self::Value) -> O + Clone,
    ) -> DelayedValDerived<O, impl Fn() -> O + Clone>
    where
        Self: Sized + Clone,
    {
        DelayedValDerived(move || func(&self.get()))
    }
}

/// For the Delayed combinator and passthroughs
pub trait DelayedParser<Input>: Parser<Input> {
    type Value;
    type DelayedValue;

    /// Obtain a handle to the output of this parser. May or may not be initialised yet.
    fn output(&self) -> Self::DelayedValue;
}
