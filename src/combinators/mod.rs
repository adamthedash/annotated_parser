//! Higher-order parsers that transform, combine, or control other parsers.
//!
//! Combinators wrap existing parsers to change their behavior, repeat them,
//! validate their output, or sequence them together. They are typically
//! constructed via [`ParserAdapter`](crate::ParserAdapter) methods or by calling the struct
//! constructors directly.

mod checkpoint;
mod conditional;
mod configured;
mod dispatch;
mod length_repeat;
mod many;
mod map;
mod optional;
mod parameterize;
mod parse_struct;
mod preceded;
mod repeat;
mod repeat_till;
mod separated;
pub(super) mod store;
mod surrounded;
mod take_till;
mod terminated;
mod trace;
mod tuple;
mod verify;

pub use checkpoint::{Checkpoint, Peek};
pub use conditional::Cond;
pub use configured::{Configured, Configuring};
pub use dispatch::Dispatch;
pub use length_repeat::LengthRepeat;
pub use many::Many;
pub use map::{Map, MapSilent, TryMap};
pub use optional::Opt;
pub use parameterize::{ParameterInput, Parameterize, Parameters};
pub use preceded::Preceded;
pub use repeat::{RepeatArray, RepeatVec};
pub use repeat_till::{RepeatTillExc, RepeatTillInc};
pub use separated::{SeparatedArray, SeparatedTuple, SeparatedVec};
pub use store::Store;
pub use surrounded::{Surrounded, SurroundedSymmetrical};
pub use take_till::{TakeTillExc, TakeTillInc};
pub use terminated::Terminated;
pub use trace::{Trace, TraceOpaque};
pub use tuple::{ParserTuple, SameParserTuple};
pub use verify::Verify;
