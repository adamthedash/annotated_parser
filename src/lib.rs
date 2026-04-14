#![cfg_attr(feature = "f16", feature(f16))]

mod adapter;
mod annotation;
pub mod combinators;
mod helpers;
mod parser;
pub mod parsers;
mod spec;

pub use adapter::ParserAdapter;
pub use annotation::{Annotation, AnnotationResult};
pub use helpers::FoldResult;
pub use parser::{IntoAnnotation, Parser, Result, SpeedyResult};
pub use parsers::ByteParser;
pub use spec::ParserSpec;
