# annotated_parser

Parser combinators with execution annotations for Rust.

This library provides parser combinators that produce inspectable execution traces, enabling both high-performance parsing and rich visualisation of parse structure.

## Why use this?

- **Inspect parser structure without running it** — [`ParserSpec`](crate::ParserSpec) mirrors the full parser hierarchy statically, useful for generating documentation, UIs, or validation.
- **Annotate execution on success or failure** — [`annotate()`](crate::Parser::annotate) collects a full trace tree for every parser in the hierarchy; [`parse()`](crate::Parser::parse) only annotates on failure, keeping the fast path lean.
- **Minimal boilerplate** — Implement one [`parse_with`](crate::Parser::parse_with) method on the [`Parser`](crate::Parser) trait and get `parse()`, `annotate()`, and all combinators for free.
- **Ergonomic combinators** — The [`ParserAdapter`](crate::ParserAdapter) trait provides `.map()`, `.repeat()`, `.many()`, `.surrounded_by()`, `.optional()`, and more, with strong type inference at the call site.
- **Good performance in non-annotating mode** — `parse()` skips annotation overhead on success paths, returning only the parsed value and consumed byte count.
- **Binary and string support** — [`ByteParser`](crate::parsers::byte::ByteParser) provides little-endian and big-endian numeric parsers for binary data; `&str` and `&[u8; N]` implement `Parser` for literal matching.

## Quick start

```rust
use annotated_parser::prelude::*;
use annotated_parser::parsers::byte::ByteParser;

// Binary: parse a little-endian u32
let mut input = &[0x01, 0x00, 0x00, 0x00][..];
let (value, bytes_consumed) = u32::LE.parse(&mut input).unwrap();
assert_eq!(value, 1);
assert_eq!(bytes_consumed, 4);

// Text: parse a literal string
let mut parser = "hello";
let mut input = "hello world";
let (value, chars_consumed) = parser.parse(&mut input).unwrap();
assert_eq!(value, "hello");
assert_eq!(input, " world");
assert_eq!(chars_consumed, 5);
```

Combinators compose via `ParserAdapter` methods:

```rust
use annotated_parser::prelude::*;

// Parse two little-endian u32s, then map the result
let mut parser = (u32::LE, u32::LE).map(|(a, b)| a + b);
let mut input = &[0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00][..];
let (value, _) = parser.parse(&mut input).unwrap();
assert_eq!(value, 3);
```

## Core concepts

- **`Parser<Input>`** — The core trait. Implement `parse_with` to define how input is consumed. The `parse()` and `annotate()` methods are provided automatically.
- **`ParserAdapter`** — A trait extension providing combinator methods (`.map()`, `.repeat()`, `.many()`, `.optional()`, `.surrounded_by()`, etc.) on any parser.
- **`Annotation` / `AnnotationResult`** — A tree structure representing the outcome of every parser in the hierarchy. Used by `annotate()` to produce full execution traces.
- **`ParserSpec`** — A static, stateless representation of the parser hierarchy. Can be inspected without running the parser on any data.

## Feature flags

- **`f16`** — Enables half-precision float parsers (`F16LE`, `F16BE`). Requires nightly Rust.

## Real-world usage

- [hex viewer](https://github.com/adamthedash/hex_viewer) - A binary file visualiser which renders the annotation tree over a hex dump.  
- [poe_data_tools](https://github.com/adamthedash/poe_data_tools) - Parsers many proprietary binary and text file formats from the game Path of Exile.  


## AI Disclaimer

LLM use has been limited to:
- Documentation
