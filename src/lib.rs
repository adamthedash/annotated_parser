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
pub use combinators::store::{
    ForwardRef, ForwardRefGet, ForwardRefTuple, ForwrdRefSet, StoringParser,
};
pub use combinators::{ParserTuple, SameParserTuple};
pub use helpers::FoldParseWithResult;
pub use parser::{
    AnnotatedResult, AnnotationMode, AnnotationReturn, IntoAnnotation, ParseResult,
    ParseWithResult, Parser, ParserOutput,
};
pub use parsers::byte::ByteParser;
pub use spec::ParserSpec;

/// Traits that usually need importing
pub mod prelude {
    pub use super::{
        ByteParser, FoldParseWithResult, ForwardRefGet, ForwardRefTuple, ForwrdRefSet,
        IntoAnnotation, Parser, ParserAdapter, ParserTuple, SameParserTuple, StoringParser,
    };
}
