use std::num::IntErrorKind;

use crate::{Annotation, Parser, ParserSpec};

macro_rules! impl_uint_parser {
    ($($name:ident => $ty:ty),* $(,)?) => {
        $(
            pub struct $name;

            impl Parser<&str> for $name {
                type Output = $ty;

                fn name(&self) -> String {
                    stringify!($ty).to_owned()
                }

                fn spec(&self) -> ParserSpec {
                    ParserSpec::empty(self.name())
                }

                fn annotate(&mut self, input: &mut &str) -> crate::AnnotatedResult<Self::Output> {
                    let end = input
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(input.len());
                    let num_chars = input[..end].chars().count();

                    let value = match input[..end].parse::<Self::Output>() {
                        Ok(v) => v,
                        Err(e) => {
                            let annotation = match e.kind() {
                                IntErrorKind::Empty => Annotation::incomplete(self.name(), 0, vec![]),
                                IntErrorKind::PosOverflow => Annotation::invalid(
                                    self.name(),
                                    0..num_chars,
                                    format!("Number doesn't fit in {}", stringify!($ty)),
                                    vec![],
                                ),
                                IntErrorKind::InvalidDigit => {
                                    unreachable!("No non-digits should reach str::parse above")
                                }
                                IntErrorKind::NegOverflow => {
                                    unreachable!("No negative digits should reach str::parse above")
                                }
                                IntErrorKind::Zero => unreachable!("Zero should be parsed properly"),
                                kind => Annotation::invalid(
                                    self.name(),
                                    0..num_chars,
                                    format!("Unknown parse error: {kind:?}"),
                                    vec![],
                                ),
                            };
                            return Err(annotation);
                        }
                    };

                    *input = &input[end..];
                    Ok((value, Annotation::success(self.name(), 0..num_chars, value, vec![])))
                }

                #[inline(always)]
                fn parse(&mut self, input: &mut &str) -> crate::ParseResult<Self::Output> {
                    let end = input
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(input.len());
                    let num_chars = input[..end].chars().count();

                    let value = match input[..end].parse::<Self::Output>() {
                        Ok(v) => v,
                        Err(e) => {
                            let annotation = match e.kind() {
                                IntErrorKind::Empty => Annotation::incomplete(self.name(), 0, vec![]),
                                IntErrorKind::PosOverflow => Annotation::invalid(
                                    self.name(),
                                    0..num_chars,
                                    format!("Number doesn't fit in {}", stringify!($ty)),
                                    vec![],
                                ),
                                IntErrorKind::InvalidDigit => {
                                    unreachable!("No non-digits should reach str::parse above")
                                }
                                IntErrorKind::NegOverflow => {
                                    unreachable!("No negative digits should reach str::parse above")
                                }
                                IntErrorKind::Zero => unreachable!("Zero should be parsed properly"),
                                kind => Annotation::invalid(
                                    self.name(),
                                    0..num_chars,
                                    format!("Unknown parse error: {kind:?}"),
                                    vec![],
                                ),
                            };
                            return Err(annotation);
                        }
                    };

                    *input = &input[end..];
                    Ok((value, num_chars))
                }
            }
        )*
    };
}
impl_uint_parser! {
    U8   => u8,
    U16  => u16,
    U32  => u32,
    U64  => u64,
    U128 => u128,
    USize   => usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good() {
        let mut input = "1234";
        let mut parser = U64;

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, 1234);
        assert_eq!(input, "");
    }

    #[test]
    fn test_leading_zeros() {
        let mut input = "001234";
        let mut parser = U64;

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, 1234);
        assert_eq!(input, "");
    }

    #[test]
    fn test_all_zeros() {
        let mut input = "00";
        let mut parser = U64;

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, 0);
        assert_eq!(input, "");
    }

    #[test]
    fn test_float() {
        let mut input = "123.01234";
        let mut parser = U64;

        let (value, _) = parser.parse(&mut input).unwrap();
        assert_eq!(value, 123);
        assert_eq!(input, ".01234");
    }

    #[test]
    fn test_empty() {
        let mut input = "";
        let mut parser = U64;

        let annotation = parser.parse(&mut input).unwrap_err();
        assert!(matches!(
            annotation.result,
            crate::AnnotationResult::Incomplete { .. }
        ));
        assert_eq!(input, "");
    }

    #[test]
    fn test_too_big() {
        let raw_input = format!("{}", u128::MAX);
        let mut input = raw_input.as_str();
        let mut parser = U64;

        let annotation = parser.parse(&mut input).unwrap_err();
        assert!(matches!(
            annotation.result,
            crate::AnnotationResult::Invalid { .. }
        ));
        assert_eq!(input, raw_input);
    }
}
