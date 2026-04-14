use crate::{
    AnnotatedResult, Annotation, FoldResult, Parser, ParserSpec,
    combinators::delayed::{DelayedValGet, DelayedValSet},
    helpers::fold_child_err,
};

/// A combinator which parameterises the inner parser with each value before running it
pub struct Parameterize<S, V, P>
where
    S: DelayedValSet,
    V: DelayedValGet<Value = Vec<S::Value>>,
    P: Parser,
{
    parameters: V,
    parameter_input: S,
    parser: P,
}

impl<S, V, P> Parameterize<S, V, P>
where
    S: DelayedValSet,
    V: DelayedValGet<Value = Vec<S::Value>>,
    P: Parser,
{
    pub fn new(parameters: V, parameter_input: S, parser: P) -> Self {
        Self {
            parameters,
            parameter_input,
            parser,
        }
    }
}

impl<S, V, P> Parser for Parameterize<S, V, P>
where
    S: DelayedValSet,
    S::Value: Clone,
    V: DelayedValGet<Value = Vec<S::Value>>,
    P: Parser,
{
    type Output = Vec<P::Output>;

    fn name(&self) -> String {
        "parameterize".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.parser.spec()])
    }

    fn annotate(&mut self, input: &mut &[u8]) -> AnnotatedResult<Self::Output> {
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

    fn parse(&mut self, input: &mut &[u8]) -> crate::ParseResult<Self::Output> {
        let parameters = self.parameters.get();

        let mut values = Vec::with_capacity(parameters.len());
        let mut offset = 0;
        for param in parameters.iter() {
            // Move this iter's param into the param slot of the parser
            self.parameter_input.set(param.clone());

            // Apply inner parser
            let value;
            (value, offset) = self
                .parser
                .parse(input)
                .map_err(|a| fold_child_err(a, vec![], offset, self.name(), 0))?;

            values.push(value);
        }

        Ok((values, offset))
    }
}
