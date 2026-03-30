use super::{DelayedValGet, DelayedValSet};
use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    ops::Deref,
    rc::Rc,
};

#[derive(Clone)]
pub struct DelayedValDerived<T, F>(pub F)
where
    F: Fn() -> T;

impl<T, F> DelayedValGet for DelayedValDerived<T, F>
where
    F: Fn() -> T,
{
    type Value = T;

    fn get(&self) -> impl Deref<Target = Self::Value> {
        let value = (self.0)();
        Box::new(value)
    }
}

/// A handle to a value which has not yet been populated. This can be used to construct parsers
/// which depend on the output of previous parsers.  
/// NOTE: Usage of this value before it has been populated is considered a compile-time parser
/// definition error, hence the use of expect rather than returning results.
pub struct DelayedVal<T>(Rc<RefCell<Option<T>>>);

impl<T> DelayedValGet for DelayedVal<T> {
    type Value = T;

    fn get(&self) -> impl Deref<Target = Self::Value> {
        let value = self.0.borrow();

        Ref::map(value, |v| v.as_ref().expect("Value has not yet been set"))
    }
}

impl<T> DelayedValSet for DelayedVal<T> {
    type Value = T;

    fn set(&self, value: Self::Value) {
        *self
            .0
            .try_borrow_mut()
            .expect("There shouldn't be any other active references to this") = Some(value);
    }

    fn take(&self) -> Self::Value {
        self.0.take().expect("Value has not yet been set")
    }
}

impl<T: Debug> Debug for DelayedVal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let binding = self.0.borrow();
        let value = binding.deref();
        write!(f, "{:?}", value)
    }
}

impl<T> Clone for DelayedVal<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Default for DelayedVal<T> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }
}
