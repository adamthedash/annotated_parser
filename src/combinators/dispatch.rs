use std::fmt::Debug;

use crate::{
    Annotation, FoldResult, Parser, ParserSpec, Result, combinators::delayed::DelayedValGet,
    helpers::fold_child_err,
};

pub struct Dispatch<const N: usize, D, F, O>
where
    D: DelayedValGet,
    F: Fn(&D::Value) -> Option<usize>,
    O: Debug,
{
    discriminant: D,
    dispatch_func: F,
    parsers: [Box<dyn Parser<Output = O>>; N],
}

impl<const N: usize, D, F, O> Dispatch<N, D, F, O>
where
    D: DelayedValGet,
    D::Value: Debug,
    F: Fn(&D::Value) -> Option<usize>,
    O: Debug,
{
    pub fn new(
        discriminant: D,
        dispatch_func: F,
        parsers: [Box<dyn Parser<Output = O>>; N],
    ) -> Self {
        Self {
            discriminant,
            dispatch_func,
            parsers,
        }
    }
}

impl<const N: usize, D, F, O> Parser for Dispatch<N, D, F, O>
where
    D: DelayedValGet,
    D::Value: Debug,
    F: Fn(&D::Value) -> Option<usize>,
    O: Debug + Clone + 'static,
{
    type Output = O;

    fn name(&self) -> String {
        "dispatch".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), self.parsers.iter().map(Parser::spec).collect())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let discriminant = self.discriminant.get();

        let Some(index) = (self.dispatch_func)(&discriminant) else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                format!("Unknown discriminant: {:?}", *discriminant),
                vec![],
            ));
        };

        let parser = self
            .parsers
            .get_mut(index)
            .expect("Dispatch function produced index out of bounds");

        let (value, span, child_annotations) =
            parser.parse(input).fold(vec![], 0, &self.name(), index)?;

        let annotation = Annotation::success(self.name(), span, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse_speedy(&mut self, input: &mut &[u8]) -> crate::SpeedyResult<Self::Output> {
        let discriminant = self.discriminant.get();

        let Some(index) = (self.dispatch_func)(&discriminant) else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                format!("Unknown discriminant: {:?}", *discriminant),
                vec![],
            ));
        };

        let parser = self
            .parsers
            .get_mut(index)
            .expect("Dispatch function produced index out of bounds");

        let (value, offset) = parser
            .parse_speedy(input)
            .map_err(|a| fold_child_err(a, vec![], 0, &self.name(), index))?;

        Ok((value, offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ByteParser;
    use crate::ParserAdapter;
    use crate::combinators::delayed::DelayedParser;

    #[test]
    fn test_dispatch() {
        let mut input = [0, 1, 1, 1, 0].as_slice();

        let disc_parser = u8::LE.delay();
        let dispatch = Dispatch::new(
            disc_parser.output(),
            |x| {
                let index = match x {
                    0 => 0,
                    1 => 1,
                    _ => return None,
                };

                Some(index)
            },
            [
                Box::new(u8::LE), //
                Box::new(u16::LE.map(|x| x as u8)),
            ],
        );

        let mut parser = (disc_parser, dispatch).repeat::<2>();
        parser.parse(&mut input).unwrap();
    }
}
