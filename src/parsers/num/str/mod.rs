use crate::AnnotationReturn;
use crate::parser::ParseWithResult;
use std::num::IntErrorKind;

use crate::{Annotation, Parser, ParserSpec};

macro_rules! impl_uint_parser {
    ($($name:ident => $ty:ty),* $(,)?) => {
        $(
            /// Parse an unsigned integer from its string representation.
            ///
            /// Consumes consecutive ASCII digits and returns the corresponding value.
            /// Fails if the input is empty or the number overflows the target type.
            ///
            /// # Example
            ///
            #[doc = concat!(
                "```\n",
                "use annotated_parser::prelude::*;\n",
                "use annotated_parser::parsers::str::", stringify!($name), ";\n",
                "\n",
                "let mut input = \"42\";\n",
                "let (value, _) = ", stringify!($name), ".parse(&mut input).unwrap();\n",
                "assert_eq!(value, 42);\n",
                "assert_eq!(input, \"\");\n",
                "```"
            )]
            pub struct $name;

            impl Parser<&str> for $name {
                type Output = $ty;

                fn name(&self) -> String {
                    stringify!($ty).to_owned()
                }

                fn spec(&self) -> ParserSpec {
                    ParserSpec::empty(self.name())
                }

                #[inline]
                fn parse_with(
                    &mut self,
                    input: &mut &str,
                    annotation_mode: crate::AnnotationMode,
                ) -> ParseWithResult<Self::Output> {
                    let end = input
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(input.len());
                    let num_chars = input[..end].chars().count();

                    let value = input[..end].parse::<Self::Output>().map_err(|e|
                        if annotation_mode.fail {
                            match e.kind(){
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
                            }.into()
                        } else {
                            AnnotationReturn::Span(0..num_chars)
                        }
                    )?;

                    *input = &input[end..];

                    let annotation = if annotation_mode.success {
                        Annotation::success(self.name(), 0..num_chars, value, vec![]).into()
                    } else {
                        AnnotationReturn::Span(0..num_chars)
                    };

                    Ok((value, annotation))

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

macro_rules! impl_int_parser {
    ($($name:ident => $ty:ty),* $(,)?) => {
        $(
            /// Parse a signed integer from its string representation.
            ///
            /// Consumes an optional leading `-` followed by ASCII digits and returns the corresponding value.
            /// Fails if the input is empty, contains only a sign, or the number overflows the target type.
            ///
            /// # Example
            ///
            #[doc = concat!(
                "```\n",
                "use annotated_parser::prelude::*;\n",
                "use annotated_parser::parsers::str::", stringify!($name), ";\n",
                "\n",
                "let mut input = \"-42\";\n",
                "let (value, _) = ", stringify!($name), ".parse(&mut input).unwrap();\n",
                "assert_eq!(value, -42);\n",
                "assert_eq!(input, \"\");\n",
                "```"
            )]
            pub struct $name;

            impl Parser<&str> for $name {
                type Output = $ty;

                fn name(&self) -> String {
                    stringify!($ty).to_owned()
                }

                fn spec(&self) -> ParserSpec {
                    ParserSpec::empty(self.name())
                }

                #[inline]
                fn parse_with(
                    &mut self,
                    input: &mut &str,
                    annotation_mode: crate::AnnotationMode,
                ) -> ParseWithResult<Self::Output> {
                    // Sign
                    let mut end = if let Some(c) = input.chars().next()
                        && c == '-'
                    {
                        1
                    } else {
                        0
                    };
                    // Digits
                    end += input[end..]
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(input[end..].len());

                    let num_chars = input[..end].chars().count();

                    let value = input[..end].parse::<Self::Output>().map_err(|e|
                        if annotation_mode.fail {
                            match e.kind() {
                                // Only InvalidDigit should be a lone "-"
                                IntErrorKind::Empty | IntErrorKind::InvalidDigit => {
                                    Annotation::incomplete(self.name(), 0, vec![])
                                }
                                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => Annotation::invalid(
                                    self.name(),
                                    0..num_chars,
                                    format!("Number doesn't fit in {}", stringify!($ty)),
                                    vec![],
                                ),
                                IntErrorKind::Zero => unreachable!("Zero should be parsed properly"),
                                kind => Annotation::invalid(
                                    self.name(),
                                    0..num_chars,
                                    format!("Unknown parse error: {kind:?}"),
                                    vec![],
                                ),
                            }.into()
                        } else {
                            AnnotationReturn::Span(0..num_chars)
                        }
                    )?;

                    *input = &input[end..];

                    let annotation = if annotation_mode.success {
                        Annotation::success(self.name(), 0..num_chars, value, vec![]).into()
                    } else {
                        AnnotationReturn::Span(0..num_chars)
                    };

                    Ok((value, annotation))
                }
            }
        )*
    };
}
impl_int_parser! {
    I8   => i8,
    I16  => i16,
    I32  => i32,
    I64  => i64,
    I128 => i128,
    ISize   => isize,
}

macro_rules! impl_float_parser {
    ($($name:ident => $ty:ty),* $(,)?) => {
        $(
            /// Parse a floating-point number from its string representation.
            ///
            /// Consumes an optional leading `-`, digits, an optional decimal point, and more digits.
            /// Fails if the input does not match a valid float format.
            ///
            /// # Example
            ///
            #[doc = concat!(
                "```\n",
                "use annotated_parser::prelude::*;\n",
                "use annotated_parser::parsers::str::", stringify!($name), ";\n",
                "\n",
                "let mut input = \"-1.5\";\n",
                "let (value, _) = ", stringify!($name), ".parse(&mut input).unwrap();\n",
                "assert_eq!(input, \"\");\n",
                "```"
            )]
            pub struct $name;

            impl Parser<&str> for $name {
                type Output = $ty;

                fn name(&self) -> String {
                    stringify!($ty).to_owned()
                }

                fn spec(&self) -> ParserSpec {
                    ParserSpec::empty(self.name())
                }

                #[inline]
                fn parse_with(
                    &mut self,
                    input: &mut &str,
                    annotation_mode: crate::AnnotationMode,
                ) -> ParseWithResult<Self::Output> {
                    // Sign
                    let mut end = if let Some(c) = input.chars().next()
                        && c == '-'
                    {
                        1
                    } else {
                        0
                    };
                    // Leading digits
                    end += input[end..]
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(input[end..].len());
                    // Dot
                    end += if let Some(c) = input[end..].chars().next()
                        && c == '.'
                    {
                        1
                    } else {
                        0
                    };
                    // Trailing digits
                    end += input[end..]
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(input[end..].len());

                    let num_chars = input[..end].chars().count();

                    let value = input[..end].parse::<Self::Output>().map_err(|e| {
                        if annotation_mode.fail {
                            Annotation::invalid(
                                self.name(),
                                0..num_chars,
                                format!("Invalid float: {e}"),
                                vec![],
                            ).into()
                        } else {
                            AnnotationReturn::Span(0..num_chars)
                        }
                    })?;

                    *input = &input[end..];

                    let annotation = if annotation_mode.success {
                        Annotation::success(self.name(), 0..num_chars, value, vec![]).into()
                    } else {
                        AnnotationReturn::Span(0..num_chars)
                    };

                    Ok((value, annotation))
                }
            }
        )*
    };
}

impl_float_parser!(
    F64 => f64,
    F32 => f32,
);

#[cfg(feature = "f16")]
impl_float_parser!(
    F16 => f16,
);

#[cfg(test)]
mod tests {
    use super::*;

    mod uint {
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

    mod int {
        use super::*;

        #[test]
        fn test_good_uint() {
            let mut input = "1234";
            let mut parser = I64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, 1234);
            assert_eq!(input, "");
        }

        #[test]
        fn test_good_int() {
            let mut input = "-1234";
            let mut parser = I64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, -1234);
            assert_eq!(input, "");
        }

        #[test]
        fn test_leading_zeros() {
            let mut input = "-001234";
            let mut parser = I64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, -1234);
            assert_eq!(input, "");
        }

        #[test]
        fn test_all_zeros() {
            let mut input = "-00";
            let mut parser = I64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, 0);
            assert_eq!(input, "");
        }

        #[test]
        fn test_float() {
            let mut input = "-123.01234";
            let mut parser = I64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, -123);
            assert_eq!(input, ".01234");
        }

        #[test]
        fn test_empty() {
            let mut input = "";
            let mut parser = I64;

            let annotation = parser.parse(&mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Incomplete { .. }
            ));
            assert_eq!(input, "");
        }

        #[test]
        fn test_empty_sign() {
            let mut input = "-";
            let mut parser = I64;

            let annotation = parser.parse(&mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Incomplete { .. }
            ));
            assert_eq!(input, "-");
        }

        #[test]
        fn test_too_big() {
            let raw_input = format!("{}", i128::MAX);
            let mut input = raw_input.as_str();
            let mut parser = I64;

            let annotation = parser.parse(&mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Invalid { .. }
            ));
            assert_eq!(input, raw_input);
        }

        #[test]
        fn test_too_small() {
            let raw_input = format!("{}", i128::MIN);
            let mut input = raw_input.as_str();
            let mut parser = I64;

            let annotation = parser.parse(&mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Invalid { .. }
            ));
            assert_eq!(input, raw_input);
        }
    }

    mod float {
        use super::*;

        #[test]
        fn test_good_uint() {
            let mut input = "1234";
            let mut parser = F64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, 1234.);
            assert_eq!(input, "");
        }

        #[test]
        fn test_good_int() {
            let mut input = "-1234";
            let mut parser = F64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, -1234.);
            assert_eq!(input, "");
        }

        #[test]
        fn test_good_float() {
            let mut input = "-123.01234";
            let mut parser = F64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, -123.01234);
            assert_eq!(input, "");
        }

        #[test]
        fn test_no_leading() {
            let mut input = "-.01234";
            let mut parser = F64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, -0.01234);
            assert_eq!(input, "");
        }

        #[test]
        fn test_no_trailing() {
            let mut input = "-123.";
            let mut parser = F64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, -123.);
            assert_eq!(input, "");
        }

        #[test]
        fn test_leading_zeros() {
            let mut input = "-001234";
            let mut parser = F64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, -1234.);
            assert_eq!(input, "");
        }

        #[test]
        fn test_all_zeros() {
            let mut input = "-00.0";
            let mut parser = F64;

            let (value, _) = parser.parse(&mut input).unwrap();
            assert_eq!(value, 0.);
            assert_eq!(input, "");
        }

        #[test]
        fn test_empty() {
            let mut input = "";
            let mut parser = F64;

            let annotation = parser.parse(&mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Invalid { .. }
            ));
            assert_eq!(input, "");
        }

        #[test]
        fn test_empty_sign() {
            let mut input = "-";
            let mut parser = F64;

            let annotation = parser.parse(&mut input).unwrap_err();
            assert!(matches!(
                annotation.result,
                crate::AnnotationResult::Invalid { .. }
            ));
            assert_eq!(input, "-");
        }
    }
}
