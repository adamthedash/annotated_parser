use itertools::izip;

use crate::{
    Annotation, AnnotationMode, AnnotationReturn, Parser, ParserSpec,
    combinators::delayed::{DelayedValGet, DelayedValGetTuple, DelayedValSet, DelayedValSetTuple},
    helpers::FoldParseWithResult,
    parser::ParseWithResult,
};

/// A combinator which parameterises the inner parser with each value before running it
pub struct Parameterize<S, V, P> {
    parameters: V,
    parameter_input: S,
    parser: P,
}

impl<S, V, P> Parameterize<S, V, P> {
    pub fn new<Input>(parameters: V, parameter_input: S, parser: P) -> Self
    where
        S: DelayedValSet,
        S::Value: Clone,
        V: DelayedValGet<Value = Vec<S::Value>>,
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
    S: DelayedValSet,
    S::Value: Clone,
    V: DelayedValGet<Value = Vec<S::Value>>,
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
        let parameters = self.parameters.get();

        let mut child_annotations = annotation_mode
            .success
            .then(|| Vec::with_capacity(parameters.len()));

        let mut values = Vec::with_capacity(parameters.len());
        let mut offset = 0;
        for param in parameters.iter() {
            // Move this iter's param into the param slot of the parser
            self.parameter_input.set(param.clone());

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

impl<P, S1, S2, V1, V2> Parameterize<(S1, S2), (V1, V2), P> {
    pub fn new_tuple<Input>(parameters: (V1, V2), parameter_input: (S1, S2), parser: P) -> Self
    where
        P: Parser<Input>,
        S1: DelayedValSet,
        S1::Value: Clone,
        V1: DelayedValGet<Value = Vec<S1::Value>>,
        S2: DelayedValSet,
        S2::Value: Clone,
        V2: DelayedValGet<Value = Vec<S2::Value>>,
    {
        Self {
            parameters,
            parameter_input,
            parser,
        }
    }
}

impl<Input, P, S1, S2, V1, V2> Parser<Input> for Parameterize<(S1, S2), (V1, V2), P>
where
    P: Parser<Input>,
    S1: DelayedValSet,
    S1::Value: Clone,
    V1: DelayedValGet<Value = Vec<S1::Value>>,
    S2: DelayedValSet,
    S2::Value: Clone,
    V2: DelayedValGet<Value = Vec<S2::Value>>,
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
        let parameters0 = self.parameters.0.get();
        let parameters1 = self.parameters.1.get();
        assert_eq!(parameters0.len(), parameters1.len());

        let mut child_annotations = annotation_mode
            .success
            .then(|| Vec::with_capacity(parameters0.len()));

        let mut values = Vec::with_capacity(parameters0.len());
        let mut offset = 0;

        let parameters = izip!(parameters0.iter(), parameters1.iter());
        for param in parameters {
            // Move this iter's param into the param slot of the parser
            self.parameter_input.0.set(param.0.clone());
            self.parameter_input.1.set(param.1.clone());

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
