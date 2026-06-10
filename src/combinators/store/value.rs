use num_traits::{PrimInt, Unsigned};
use paste::paste;
use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    rc::Rc,
};

/// Read access to a lazily-populated value.
///
/// Implementors provide a `try_get` method that returns `None` if the value
/// has not been set yet. This is the read-side of the forward-reference system.
pub trait ForwardRefGet {
    /// Type of the stored value.
    type Value;

    /// Get a reference to the currently stored value.
    ///
    /// Panics if the value has not yet been set.
    fn get(&self) -> Ref<'_, Self::Value> {
        Ref::map(self.try_get(), |v| {
            v.as_ref().expect("Value has not yet been set")
        })
    }

    /// Get a reference to the currently stored value, or `None` if not yet set.
    fn try_get(&self) -> Ref<'_, Option<Self::Value>>;
}

/// Write access to a forward-reference value.
///
/// Implementors provide a `set` method to populate the value. This is the
/// write-side of the forward-reference system.
pub trait ForwrdRefSet {
    /// Type of the stored value.
    type Value;

    /// Set or overwrite the stored value.
    fn set(&self, value: Self::Value);
}

// =====================================================================

/// A shared, lazily-initialised value that can be read before it is written.
///
/// `ForwardRef` is the core of the forward-reference system. It is used to
/// pass values from one parser to another without needing to know the parse
/// order at compile time. Values are typically set by [`Store`](super::Store) and read by
/// combinators like [`Cond`](crate::combinators::Cond), [`RepeatVec`](crate::combinators::RepeatVec), and [`Dispatch`](crate::combinators::Dispatch).
///
/// There are two kinds:
/// - **Source**: set directly by a parser, created via `new_source()` or `with_value()`.
/// - **Derived**: computed from other `ForwardRef`s, created via `new_derived()` or `map()`.
pub struct ForwardRef<T>(ForwardRefInner<T>);

impl<T: 'static> ForwardRef<T> {
    /// Create a new uninitialised source value.
    pub fn new_source() -> Self {
        Self(ForwardRefInner::Source(SourceValue::default()))
    }

    /// Create a new source value initialised with the given value.
    pub fn with_value(value: T) -> Self {
        Self(ForwardRefInner::Source(SourceValue::with_value(value)))
    }

    /// Create a new derived value with the given value generation function.
    ///
    /// The function should return `None` if the value cannot be generated.
    pub fn new_derived<F>(func: F) -> Self
    where
        F: Fn() -> Option<T> + 'static,
    {
        Self(ForwardRefInner::Derived(Rc::new(DerivedValue::new(func))))
    }

    /// Create a new derived value by mapping the value of this one.
    pub fn map<F, O>(&self, func: F) -> ForwardRef<O>
    where
        F: Fn(&T) -> O + 'static,
        O: 'static,
    {
        let value = self.clone();
        ForwardRef::new_derived(move || {
            let borrow = value.try_get();
            // If upstream value is None, then this is also None
            let value = borrow.as_ref()?;
            let out = func(value);
            Some(out)
        })
    }

    /// Check if the given bit is set.
    pub fn bitflag(&self, bit: usize) -> ForwardRef<bool>
    where
        T: PrimInt + Unsigned,
    {
        self.map(move |flags| (*flags >> bit) & T::one() == T::one())
    }
}

impl<T> Clone for ForwardRef<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Debug> Debug for ForwardRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ForwardRefInner::Source(source) = &self.0 else {
            unreachable!("Derived values shouldn't ever be displayed");
        };

        let r = source.0.borrow();
        let Some(value) = r.as_ref() else {
            unreachable!("Un-initialised references shouldn't ever be displayed");
        };

        value.fmt(f)
    }
}

impl<T> ForwardRefGet for ForwardRef<T> {
    type Value = T;

    fn try_get(&self) -> Ref<'_, Option<Self::Value>> {
        self.0.try_get()
    }
}

impl<T> ForwrdRefSet for ForwardRef<T> {
    type Value = T;

    fn set(&self, value: Self::Value) {
        self.0.set(value);
    }
}

// =====================================================================

enum ForwardRefInner<T> {
    /// Raw value usually produced by a parser. Can be set or get
    Source(SourceValue<T>),
    /// Value derived from others, materialised on request. Only get
    Derived(Rc<DerivedValue<T>>),
}

impl<T> Clone for ForwardRefInner<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Source(val) => Self::Source(val.clone()),
            Self::Derived(val) => Self::Derived(val.clone()),
        }
    }
}

impl<T> ForwardRefGet for ForwardRefInner<T> {
    type Value = T;

    fn try_get(&self) -> Ref<'_, Option<Self::Value>> {
        match self {
            ForwardRefInner::Source(value) => value.try_get(),
            ForwardRefInner::Derived(value) => value.try_get(),
        }
    }
}

impl<T> ForwrdRefSet for ForwardRefInner<T> {
    type Value = T;

    fn set(&self, value: Self::Value) {
        let Self::Source(val) = self else {
            panic!("Only Source values are settable");
        };

        val.set(value);
    }
}

// =====================================================================

/// Value which is manually set by a parser
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

impl<T> ForwardRefGet for SourceValue<T> {
    type Value = T;

    fn try_get(&self) -> Ref<'_, Option<Self::Value>> {
        self.0.borrow()
    }
}

impl<T> ForwrdRefSet for SourceValue<T> {
    type Value = T;

    fn set(&self, value: Self::Value) {
        *self.0.borrow_mut() = Some(value);
    }
}

// =====================================================================

/// Value which is generated from other values
struct DerivedValue<T> {
    /// Cache result internally so lifetime is bound to self instead of get()
    value: RefCell<Option<T>>,
    func: Box<dyn Fn() -> Option<T>>,
}

impl<T> DerivedValue<T> {
    fn new<F>(func: F) -> Self
    where
        F: Fn() -> Option<T> + 'static,
    {
        Self {
            value: RefCell::default(),
            func: Box::new(func),
        }
    }
}

impl<T> ForwardRefGet for DerivedValue<T> {
    type Value = T;

    fn try_get(&self) -> Ref<'_, Option<Self::Value>> {
        // Compute & cache
        *self.value.borrow_mut() = (self.func)();

        self.value.borrow()
    }
}

// =====================================================================

/// Tuple of `ForwardRef`s that can be read together.
///
/// Implemented for tuples of `ForwardRef` so that multiple references can be
/// dereferenced in a single call. Also provides `map()` for combining multiple
/// references into a derived value.
pub trait ForwardRefTuple {
    /// Tuple of references to the stored values.
    type Ref<'a>
    where
        Self: 'a;

    /// Get all references. Panics if any value has not yet been set.
    fn get_tuple(&self) -> Self::Ref<'_>;

    /// Get all references, returning `None` if any value has not yet been set.
    fn try_get_tuple(&self) -> Option<Self::Ref<'_>>;

    /// Create a derived `ForwardRef` by applying a function to all references.
    fn map<F, O>(self, func: F) -> ForwardRef<O>
    where
        F: Fn(Self::Ref<'_>) -> O + 'static,
        O: 'static,
        Self: Sized + 'static,
    {
        ForwardRef::new_derived(move || {
            let refs = self.try_get_tuple()?;
            let out = func(refs);
            Some(out)
        })
    }
}

macro_rules! impl_tuple {
    ($($idx:tt, )*) => {
    paste!{
        impl<$([<T $idx>],)*> ForwardRefTuple for ($(ForwardRef<[<T $idx>]>, )*)
        where
            $(
                [<T $idx>]: 'static,
            )*
        {
            type Ref<'a> = ($(Ref<'a, [<T $idx>]>,)*);

            fn get_tuple(&self) -> Self::Ref<'_> {
                ($(self.$idx.get(),)*)
            }

            fn try_get_tuple(&self) -> Option<Self::Ref<'_>> {
                let refs = ($({
                    let r = self.$idx.try_get();
                    if r.is_none() {
                        return None;
                    }
                    Ref::map(r, |r| r.as_ref().expect("Checked above"))
                },)*);

                Some(refs)
            }
        }
    }
    };
}

impl_tuple!(0,);
impl_tuple!(0, 1,);
impl_tuple!(0, 1, 2,);
impl_tuple!(0, 1, 2, 3,);
impl_tuple!(0, 1, 2, 3, 4,);
impl_tuple!(0, 1, 2, 3, 4, 5,);
impl_tuple!(0, 1, 2, 3, 4, 5, 6,);
impl_tuple!(0, 1, 2, 3, 4, 5, 6, 7,);
impl_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8,);
impl_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9,);
impl_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,);
impl_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,);
