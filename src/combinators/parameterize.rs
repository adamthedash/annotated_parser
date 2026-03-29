use std::{fmt::Debug, ops::Deref};

use crate::{
    Annotation, FoldResult, Parser, ParserSpec, Result,
    combinators::delayed::{DelayedValGet, DelayedValSet},
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
    P::Output: Debug,
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
    P::Output: Debug,
{
    type Output = Vec<P::Output>;

    fn name(&self) -> String {
        "parameterize".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.parser.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let name = self.name();
        let (values, span, child_annotations) = self.parameters.get().deref().iter().try_fold(
            (vec![], 0..0, vec![]),
            |(mut out_values, out_span, child_annotations), value| {
                // Move this iter's param into the param slot of the parser
                self.parameter_input.set(value.clone());

                // Apply inner parser
                let (out_value, span, child_annotations) =
                    self.parser
                        .parse(input)
                        .fold(child_annotations, out_span.end, &name, 0)?;

                out_values.push(out_value);

                Ok((out_values, 0..span.end, child_annotations))
            },
        )?;

        let annotation = Annotation::success(&self.name(), span, &values, child_annotations);

        Ok((values, annotation))
    }
}
