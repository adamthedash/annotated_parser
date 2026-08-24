#![cfg_attr(feature = "f16", feature(f16))]

mod adapter;
mod annotation;
pub mod combinators;
pub mod helpers;
mod parser;
pub mod parsers;
mod spec;

use std::sync::atomic::AtomicUsize;

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

/// Limit on the number of items that an be parsed by any dynamically sized parser
pub static ALLOC_LIMIT: AtomicUsize = AtomicUsize::new(1_000_000);

/// Traits that usually need importing
pub mod prelude {
    pub use super::{
        ByteParser, FoldParseWithResult, ForwardRefGet, ForwardRefTuple, ForwrdRefSet,
        IntoAnnotation, Parser, ParserAdapter, ParserTuple, SameParserTuple, StoringParser,
    };
}
