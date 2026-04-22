use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    rc::Rc,
};

pub trait DelayedValGet {
    type Value;

    /// Get a ref to the currently stored value
    fn get(&self) -> Ref<'_, Self::Value>;
}

pub trait DelayedValSet {
    type Value;

    /// Set/overwrite the stored value
    fn set(&self, value: Self::Value);

    /// Take the stored value, un-setting the storage
    fn take(&self) -> Self::Value;
}

// =====================================================================

pub struct DelayedVal<T>(DelayedValInner<T>);

impl<T: 'static> DelayedVal<T> {
    /// Create a new uninitialised source value
    pub fn new_source() -> Self {
        Self(DelayedValInner::Source(SourceValue::default()))
    }

    /// Create a new source value initialised with the given value
    pub fn with_value(value: T) -> Self {
        Self(DelayedValInner::Source(SourceValue::with_value(value)))
    }

    /// Create a new derived value with the given value generation function
    pub fn new_derived<F>(func: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        Self(DelayedValInner::Derived(Rc::new(DerivedValue::new(func))))
    }

    /// Create a new derived value by mapping the value of this one
    pub fn map<F, O>(&self, func: F) -> DelayedVal<O>
    where
        F: Fn(&T) -> O + 'static,
        O: 'static,
    {
        let value = self.clone();
        DelayedVal::new_derived(move || func(&value.get()))
    }
}

impl<T> Clone for DelayedVal<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Debug> Debug for DelayedVal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> DelayedValGet for DelayedVal<T> {
    type Value = T;

    fn get(&self) -> Ref<'_, Self::Value> {
        self.0.get()
    }
}

impl<T> DelayedValSet for DelayedVal<T> {
    type Value = T;

    fn set(&self, value: Self::Value) {
        self.0.set(value);
    }

    fn take(&self) -> Self::Value {
        self.0.take()
    }
}

// =====================================================================

enum DelayedValInner<T> {
    /// Raw value usually produced by a parser. Can be set or get
    Source(SourceValue<T>),
    /// Value derived from others, materialised on request. Only get
    Derived(Rc<DerivedValue<T>>),
}

impl<T: Debug> Debug for DelayedValInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(val) => f.debug_tuple("Source").field(val).finish(),
            Self::Derived(_val) => f.debug_tuple("Derived").finish(),
        }
    }
}

impl<T> Clone for DelayedValInner<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Source(val) => Self::Source(val.clone()),
            Self::Derived(val) => Self::Derived(val.clone()),
        }
    }
}

impl<T> DelayedValGet for DelayedValInner<T> {
    type Value = T;

    fn get(&self) -> Ref<'_, Self::Value> {
        match self {
            DelayedValInner::Source(value) => value.get(),
            DelayedValInner::Derived(value) => value.get(),
        }
    }
}

impl<T> DelayedValSet for DelayedValInner<T> {
    type Value = T;

    fn set(&self, value: Self::Value) {
        let Self::Source(val) = self else {
            panic!("Only Source values are settable");
        };

        val.set(value);
    }

    fn take(&self) -> Self::Value {
        let Self::Source(val) = self else {
            panic!("Only Source values are takeable");
        };

        val.take()
    }
}

// =====================================================================

/// Value which is manually set by a parser
#[derive(Debug)]
struct SourceValue<T>(Rc<RefCell<Option<T>>>);

impl<T> SourceValue<T> {
    fn with_value(value: T) -> Self {
        Self(Rc::new(RefCell::new(Some(value))))
    }
}

impl<T> Default for SourceValue<T> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }
}

impl<T> Clone for SourceValue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> DelayedValGet for SourceValue<T> {
    type Value = T;

    fn get(&self) -> Ref<'_, Self::Value> {
        Ref::map(self.0.borrow(), |value| {
            value.as_ref().expect("Value has not yet been set")
        })
    }
}

impl<T> DelayedValSet for SourceValue<T> {
    type Value = T;

    fn set(&self, value: Self::Value) {
        *self.0.borrow_mut() = Some(value);
    }

    fn take(&self) -> Self::Value {
        self.0.take().expect("Take on None")
    }
}

// =====================================================================

/// Value which is generated from other values
struct DerivedValue<T> {
    /// Cache result internally so lifetime is bound to self instead of get()
    value: RefCell<Option<T>>,
    func: Box<dyn Fn() -> T>,
}

impl<T> DerivedValue<T> {
    fn new<F>(func: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        Self {
            value: RefCell::default(),
            func: Box::new(func),
        }
    }
}

impl<T> DelayedValGet for DerivedValue<T> {
    type Value = T;

    fn get(&self) -> Ref<'_, Self::Value> {
        // Compute & cache
        let value = (self.func)();
        *self.value.borrow_mut() = Some(value);

        Ref::map(self.value.borrow(), |value| {
            value.as_ref().expect("Value has not yet been set")
        })
    }
}
