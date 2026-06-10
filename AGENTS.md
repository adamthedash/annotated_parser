# AGENTS.md

## Project

A parser combinator library with execution annotations. Rust edition 2024.

## Build & Test

- **Standard Cargo**: `cargo test`, `cargo build`, `cargo doc`.
- **Nightly feature**: The `f16` feature requires nightly Rust (`#![feature(f16)]`). `cargo test --all-features` will fail on stable.
- **No CI, no formatter/linter config, no task runner.**

## Architecture

- `src/parser.rs` — Core `Parser<Input>` trait. `parse()` is the fast path (annotations on failure only); `annotate()` collects full annotations.
- `src/adapter.rs` — `ParserAdapter` trait with combinator methods (`.map()`, `.repeat()`, `.many()`, etc.).
- `src/annotation.rs` — `Annotation` / `AnnotationResult` tree for parse visualization.
- `src/spec.rs` — `ParserSpec` for static parser structure inspection.
- `src/parsers/` — Leaf parsers. Primitive byte parsers are obtained via the `ByteParser` trait (e.g. `u32::LE`). `&[u8; N]` and `&str` also implement `Parser` as literal matchers.
- `src/combinators/` — Parser combinators. `Store` is a combinator (in `src/combinators/store/`) that captures parser output in a `ForwardRef`.
- `src/helpers.rs` — `FoldParseWithResult` trait for accumulating child parser results when writing custom combinators.
- `src/lib.rs` — Exports `prelude` module as the intended import path.

## Conventions

- `Parser` implementors return `AnnotationReturn` from `parse_with`. `AnnotationReturn` is public-facing for custom parser authors.
- `ParserOutput` is a blanket trait (`Debug + Clone + Send + Sync + 'static`) for values stored in annotations.
- `ForwardRef` / `ForwardRefGet` in `src/combinators/store/` provide runtime-forwarded values for dynamic-length parsers.
- Tuple parsers (`impl Parser for (A, B, C)`) are macro-generated up to 12-tuples via `paste`. Do not modify or document these directly.
- Tests are unit tests embedded in source files under `#[cfg(test)]`. No `tests/` directory.
- `jj` (jujutsu) is used alongside git (`.jj/` directory exists).

## Rules

Before making any changes to the codebase and during planning, check relevant `.agents/rules/*.md` files to ensure the correct conventions are being used.
