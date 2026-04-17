use crate::{
    AnnotatedResult, Annotation, AnnotationMode, AnnotationReturn, FoldAnnotatedResult, Parser,
    ParserSpec,
    combinators::delayed::{DelayedValGet, DelayedValSet},
    helpers::{FoldParseResult, FoldParseWithResult},
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

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        let parameters = self.parameters.get();

        let mut child_annotations = Vec::with_capacity(parameters.len());
        let mut values = Vec::with_capacity(parameters.len());
        let mut offset = 0;
        for param in parameters.iter() {
            // Move this iter's param into the param slot of the parser
            self.parameter_input.set(param.clone());

            // Apply inner parser
            let value;
            (value, offset, child_annotations) =
                self.parser
                    .annotate(input)
                    .fold(child_annotations, offset, || self.name(), 0)?;

            values.push(value);
        }

        let annotation =
            Annotation::success(self.name(), 0..offset, values.clone(), child_annotations);

        Ok((values, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let parameters = self.parameters.get();

        let mut values = Vec::with_capacity(parameters.len());
        let mut offset = 0;
        for param in parameters.iter() {
            // Move this iter's param into the param slot of the parser
            self.parameter_input.set(param.clone());

            // Apply inner parser
            let value;
            (value, offset) = self.parser.parse(input).fold(offset, || self.name(), 0)?;

            values.push(value);
        }

        Ok((values, offset))
    }
}
