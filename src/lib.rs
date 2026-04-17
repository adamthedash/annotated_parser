#![cfg_attr(feature = "f16", feature(f16))]

mod adapter;
mod annotation;
pub mod combinators;
pub mod helpers;
mod parser;
pub mod parsers;
mod spec;

pub use adapter::ParserAdapter;
pub use annotation::{Annotation, AnnotationResult};
pub use helpers::{FoldAnnotatedResult, FoldParseWithResult};
pub use parser::{
    AnnotatedResult, AnnotationMode, AnnotationReturn, IntoAnnotation, ParseResult,
    ParseWithResult, Parser,
};
pub use parsers::byte::ByteParser;
pub use spec::ParserSpec;

/// Traits that usually need importing
pub mod prelude {
    pub use super::{ByteParser, FoldParseWithResult, IntoAnnotation, Parser, ParserAdapter};
}
