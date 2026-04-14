use std::fmt::Debug;

use crate::{
    AnnotatedResult, Annotation, FoldResult, Parser, ParserSpec,
    combinators::delayed::DelayedValGet, helpers::fold_child_err,
};

pub struct Dispatch<const N: usize, I, D, F, O> {
    discriminant: D,
    dispatch_func: F,
    parsers: [Box<dyn for<'a> Parser<'a, Input = I, Output = O>>; N],
}

impl<const N: usize, I, D, F, O> Dispatch<N, I, D, F, O>
where
    D: DelayedValGet,
    D::Value: Debug,
    F: Fn(&D::Value) -> Option<usize>,
    O: Debug,
{
    pub fn new(
        discriminant: D,
        dispatch_func: F,
        parsers: [Box<dyn for<'a> Parser<'a, Input = I, Output = O>>; N],
    ) -> Self {
        Self {
            discriminant,
            dispatch_func,
            parsers,
        }
    }
}

impl<'a, const N: usize, I, D, F, O> Parser<'a> for Dispatch<N, I, D, F, O>
where
    I: Copy + 'a,
    D: DelayedValGet,
    D::Value: Debug,
    F: Fn(&D::Value) -> Option<usize>,
    O: Debug + Clone + 'static,
{
    type Input = I;

    type Output = O;

    fn name(&self) -> String {
        "dispatch".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), self.parsers.iter().map(Parser::spec).collect())
    }

    fn annotate(&mut self, input: &mut I) -> AnnotatedResult<Self::Output> {
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

        let (value, offset, child_annotations) =
            parser
                .annotate(input)
                .fold(vec![], 0, || self.name(), index)?;

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut I) -> crate::ParseResult<Self::Output> {
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
            .parse(input)
            .map_err(|a| fold_child_err(a, vec![], 0, self.name(), index))?;

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
        parser.annotate(&mut input).unwrap();
    }
}
