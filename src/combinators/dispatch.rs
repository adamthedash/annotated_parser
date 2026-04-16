use crate::FoldResult;
use crate::combinators::tuple::{ParserTuple, SameParserTuple};

use crate::{
    AnnotatedResult, Annotation, Parser, ParserSpec, combinators::delayed::DelayedValGet,
    helpers::fold_child_err,
};

pub struct Dispatch<D, P> {
    discriminant: D,
    parsers: P,
}

impl<D, P> Dispatch<D, P>
where
    D: DelayedValGet<Value = Option<usize>>,
{
    pub fn new<Input>(discriminant: D, parsers: P) -> Self
    where
        P: ParserTuple<Input>,
    {
        Self {
            discriminant,
            parsers,
        }
    }
}

impl<Input, D, P> Parser<Input> for Dispatch<D, P>
where
    D: DelayedValGet<Value = Option<usize>>,
    P: SameParserTuple<Input>,
{
    type Output = P::Output;

    fn name(&self) -> String {
        "dispatch".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), self.parsers.specs())
    }

    fn annotate(&mut self, input: &mut Input) -> AnnotatedResult<Self::Output> {
        let Some(index) = *self.discriminant.get() else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                "Unknown discriminant".to_string(),
                vec![],
            ));
        };

        let Some(res) = self.parsers.annotate(input, index) else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                "Discriminant out of bounds".to_string(),
                vec![],
            ));
        };

        let (value, offset, child_annotations) = res.fold(vec![], 0, || self.name(), index)?;

        let annotation =
            Annotation::success(self.name(), 0..offset, value.clone(), child_annotations);

        Ok((value, annotation))
    }

    fn parse(&mut self, input: &mut Input) -> crate::ParseResult<Self::Output> {
        let Some(index) = *self.discriminant.get() else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                "Unknown discriminant".to_string(),
                vec![],
            ));
        };

        let Some(res) = self.parsers.parse(input, index) else {
            return Err(Annotation::invalid(
                self.name(),
                0..0,
                "Discriminant out of bounds".to_string(),
                vec![],
            ));
        };

        let (value, offset) = res.map_err(|a| fold_child_err(a, vec![], 0, self.name(), index))?;

        Ok((value, offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ByteParser;
    use crate::ParserAdapter;
    use crate::combinators::delayed::DelayedParser;
    use crate::combinators::delayed::DelayedVal;

    #[test]
    fn test_dispatch() {
        let mut input = [0, 1, 1, 1, 0].as_slice();

        let disc_parser = u8::LE.delay();
        let dispatch = Dispatch::new(
            disc_parser.output().map(|x| {
                let index = match x {
                    0 => 0,
                    1 => 1,
                    _ => return None,
                };

                Some(index)
            }),
            (
                u8::LE, //
                u16::LE.map(|x| x as u8),
            ),
        );

        let mut parser = (disc_parser, dispatch).repeat::<2>();
        parser.annotate(&mut input).unwrap();
    }

    #[test]
    fn test_dispatch2() {
        fn create_parser() -> impl for<'a> Parser<&'a [u8], Output = u8> {
            Dispatch::new(
                DelayedVal::with_value(Some(0)),
                (
                    u8::LE, //
                    u16::LE.map(|x| x as u8),
                ),
            )
        }

        fn use_parser() -> (Vec<u8>, u8) {
            let mut parser = create_parser();

            let input = vec![0; 5];
            let (value, _) = parser.parse(&mut input.as_slice()).unwrap();

            (input, value)
        }

        use_parser();
    }
}
