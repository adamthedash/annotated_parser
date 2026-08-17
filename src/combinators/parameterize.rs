use itertools::izip;
use paste::paste;

use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec,
    combinators::store::{ForwardRef, ForwardRefGet, ForwrdRefSet},
    helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

// ==================================================================

/// A set of parameters which are used to configure a parser
#[allow(clippy::len_without_is_empty)]
pub trait Parameters {
    /// Item for a single parameter combination
    type Item;

    /// Iterate over parameter combinations
    fn iter(&self) -> impl Iterator<Item = Self::Item>;

    /// Total parameter combinations
    fn len(&self) -> usize;
}

impl<T> Parameters for ForwardRef<Vec<T>>
where
    T: Clone,
{
    type Item = T;

    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        self.get().clone().into_iter()
    }

    fn len(&self) -> usize {
        self.get().len()
    }
}

// Separate impl for 1-tuple as izip doesn't work with single iterables
impl<T> Parameters for (ForwardRef<Vec<T>>,)
where
    T: Clone,
{
    type Item = (T,);

    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        // NOTE: Clone entire vec upfront as we need to clone anyway to store it in the temp
        // location. This helps with lifetimes.
        self.0.get().clone().into_iter().map(|x| (x,))
    }

    fn len(&self) -> usize {
        self.0.get().len()
    }
}

// ==================================================================

/// Something that can serve as a temporary place to set a parser variable. Eg. the count of a
/// RepeatVec.
pub trait ParameterInput {
    type Value;

    /// Move the provided value into the temp slot
    fn set_temp(&self, value: Self::Value);
}

impl<T> ParameterInput for ForwardRef<T> {
    type Value = T;

    fn set_temp(&self, value: Self::Value) {
        self.set(value);
    }
}

impl<T> ParameterInput for (ForwardRef<T>,) {
    type Value = (T,);

    fn set_temp(&self, value: Self::Value) {
        self.0.set(value.0);
    }
}

// ==================================================================

/// Parameters, ParameterInput for tuples of ForwardRef's
macro_rules! impl_parameters {
    ($($idx:tt, )*) => {
    paste!{
        impl<$([<T $idx>],)*> Parameters for ($(ForwardRef<Vec<[<T $idx>]>>, )*)
        where
            $(
                [<T $idx>]: Clone,
            )*
        {
            type Item = ($([<T $idx>],)*);

            fn iter(&self) -> impl Iterator<Item = Self::Item> {
                // NOTE: Clone entire vec upfront as we need to clone anyway to store it in the temp
                // location. This helps with lifetimes.
                $(
                    let [<p $idx>] = self.$idx.get().clone();
                )*

                izip!($([<p $idx>], )*)
            }

            fn len(&self) -> usize {
                // NOTE: All parameters should be the same length.
                // TODO: Check all and panic if they're not?
                self.0.len()
            }
        }

        impl<$([<T $idx>],)*> ParameterInput for ($(ForwardRef<[<T $idx>]>, )*) {
            type Value = ($([<T $idx>],)*);

            fn set_temp(&self, value: Self::Value) {
                $(
                    self.$idx.set(value.$idx);
                )*
            }
        }
    }
    };
}

impl_parameters!(0, 1,);
impl_parameters!(0, 1, 2,);
impl_parameters!(0, 1, 2, 3,);
impl_parameters!(0, 1, 2, 3, 4,);
impl_parameters!(0, 1, 2, 3, 4, 5,);
impl_parameters!(0, 1, 2, 3, 4, 5, 6,);
impl_parameters!(0, 1, 2, 3, 4, 5, 6, 7,);
impl_parameters!(0, 1, 2, 3, 4, 5, 6, 7, 8,);
impl_parameters!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9,);
impl_parameters!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,);
impl_parameters!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,);

// ==================================================================

/// Repeat a parser with different parameter values each time.
///
/// Iterates over the provided parameters, sets each one into a `ParameterInput` slot,
/// then runs the inner parser. Collects all results into a `Vec`.
///
/// This is useful for running the same parser shape with varying arguments,
/// such as different expected magic numbers or lengths.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::combinators::Parameterize;
/// use annotated_parser::ForwardRef;
/// use annotated_parser::parsers::TakeVec;
///
/// let params = ForwardRef::with_value(vec![2usize, 3, 1]);
/// let slot = ForwardRef::new_source();
/// let mut chunks = Parameterize::new::<&[u8]>(params, slot.clone(), TakeVec::new(slot));
///
/// let mut input = &[10, 11, 20, 21, 22, 30][..];
/// let (chunks, _) = chunks.parse(&mut input).unwrap();
/// assert_eq!(chunks, vec![vec![10, 11], vec![20, 21, 22], vec![30]]);
/// ```
pub struct Parameterize<S, V, P> {
    parameters: V,
    parameter_input: S,
    parser: P,
}

impl<S, V, P> Parameterize<S, V, P> {
    pub fn new<Input>(parameters: V, parameter_input: S, parser: P) -> Self
    where
        V: Parameters,
        S: ParameterInput<Value = V::Item>,
        P: Parser<Input>,
    {
        Self {
            parameters,
            parameter_input,
            parser,
        }
    }
}

impl<Input, S, V, P> Parser<Input> for Parameterize<S, V, P>
where
    V: Parameters,
    S: ParameterInput<Value = V::Item>,
    P: Parser<Input>,
{
    type Output = Vec<P::Output>;

    fn name(&self) -> String {
        "parameterize".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.parser.spec()])
    }

    fn parse_with(
        &mut self,
        input: &mut Input,
        annotation_mode: AnnotationMode,
    ) -> ParseWithResult<Self::Output> {
        // let parameters = self.parameters.get();

        let num_params = self.parameters.len();
        let mut child_annotations = annotation_mode
            .success
            .then(|| Vec::with_capacity(num_params));

        let mut values = Vec::with_capacity(num_params);
        let mut offset = 0;
        for param in self.parameters.iter() {
            // Move this iter's param into the param slot of the parser
            self.parameter_input.set_temp(param);

            // Apply inner parser
            let value;
            (value, offset, child_annotations) =
                self.parser.parse_with(input, annotation_mode).fold(
                    annotation_mode,
                    || self.name(),
                    child_annotations,
                    offset,
                    0,
                )?;

            values.push(value);
        }

        let annotation = if annotation_mode.success {
            Annotation::success(
                self.name(),
                0..offset,
                values.clone(),
                child_annotations.unwrap(),
            )
            .into()
        } else {
            AnnotationReturn::Span(0..offset)
        };

        Ok((values, annotation))
    }
}
