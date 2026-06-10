# Parser-Level Documentation

## Scope

- Add `///` doc comments **only on parser structs** (the types that implement `Parser`).
- Do not add module-level docs, file-level docs, or README updates unless explicitly requested.

## Content

Each parser struct doc comment should contain:

1. **A short description** (2–3 sentences) of what the parser does, what it consumes, and when it fails.
2. **One minimal usage example** in a `/// ```rust` (or `/// ```ignore` for nightly-only code) block showing how to call the parser.

## Example Style

```rust
/// Parse a value from its little-endian byte representation.
///
/// Consumes exactly `N` bytes from the input and interprets them as a little-endian
/// value of type `T`, where `N` is the byte size of `T`. Fails if the input is too short.
///
/// # Example
///
/// ```
/// use annotated_parser::prelude::*;
/// use annotated_parser::parsers::byte::ByteParser;
///
/// let mut input = &[0x01, 0x00, 0x00, 0x00][..];
/// let (value, _) = u32::LE.parse(&mut input).unwrap();
/// assert_eq!(value, 1);
/// ```
```

## Macro Doc Tests

If the parser is generated inside a `macro_rules!` macro, macro metavariables like `$name` are **not expanded** inside `///` doc test blocks. Use `#[doc = concat!(...)]` with `stringify!` instead:

```rust
#[doc = concat!(
    "/// Parse a `", stringify!($ty), "` from ...\n",
    "///\n",
    "/// # Example\n",
    "///\n",
    "```\n",
    "use annotated_parser::prelude::*;\n",
    "use annotated_parser::parsers::str::", stringify!($name), ";\n",
    "\n",
    "let mut input = \"42\";\n",
    "let (value, _) = ", stringify!($name), ".parse(&mut input).unwrap();\n",
    "assert_eq!(value, 42);\n",
    "```"
)]
```

## What Not to Document

- Do not add `///` docs to private helpers, macro internals, or combinator types.
- Do not change the `parsers/mod.rs` re-export structure.
- Do not add module-level or crate-level docs.
