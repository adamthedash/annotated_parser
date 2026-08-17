//! Primitive parsers that consume raw input directly.
//!
//! Leaf parsers are the building blocks of the parser hierarchy. Unlike
//! combinators, they do not wrap other parsers; they interact with the input
//! directly.
//!
//! For binary data, primitive byte parsers are obtained through the
//! [`ByteParser`](byte::ByteParser) trait, which is implemented automatically for types that
//! implement `FromBytes` (e.g. `u32::LE`, `u64::BE`). The `byte` and `str`
//! submodules provide additional numeric parsers for binary and string inputs.
//!
//! In addition to the exported types, the following types also implement
//! [`Parser`](crate::Parser):
//! - `&[u8; N]` — literal byte array matching
//! - `&str` — literal string matching

mod empty;
mod eof;
mod literal;
mod num;
mod skip;
mod take;

pub use empty::Empty;
pub use eof::EoF;
pub use num::{byte, str};
pub use skip::{SkipArray, SkipVec};
pub use take::{TakeArray, TakeVec};
