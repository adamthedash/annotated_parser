# AGENTS.md

## Project

A parser combinator library with execution annotations.

## Build & Test

- **Standard Cargo**: `cargo test`, `cargo build`, `cargo doc`.
- **Nightly feature**: The `f16` feature requires nightly Rust (`#![feature(f16)]`). `cargo test --all-features` will fail on stable.
- **No CI, no formatter/linter config, no task runner.**

## Architecture

- `src/parser.rs` — Core `Parser<Input>` trait. `parse()` is the fast path (annotations on failure only); `annotate()` collects full annotations.
- `src/adapter.rs` — `ParserAdapter` trait with combinator methods (`.map()`, `.repeat()`, `.many()`, etc.).
- `src/annotation.rs` — `Annotation` / `AnnotationResult` tree for parse visualization.
- `src/spec.rs` — `ParserSpec` for static parser structure inspection.
- `src/parsers/` — Leaf parsers (literals, numbers, EOF, empty, take).
- `src/combinators/` — Parser combinators (map, repeat, optional, surrounded, etc.).
- `src/lib.rs` — Exports `prelude` module as the intended import path.

## Conventions

- `Parser` implementors return `AnnotationReturn` from `parse_with`. `AnnotationReturn` is an internal type; callers use `parse()` or `annotate()`.
- `ParserOutput` is a blanket trait (`Debug + Clone + Send + Sync + 'static`) for values stored in annotations.
- `ForwardRef` / `ForwardRefGet` in `src/combinators/store/` provide runtime-forwarded values for dynamic-length parsers.
- Tests are unit tests embedded in source files under `#[cfg(test)]`. No `tests/` directory.
- `jj` (jujutsu) is used alongside git (`.jj/` directory exists).

# Rules
Before making any changes to the codebase and during planning, check relevant `.agents/rules/*.md` files to ensure the correct conventions are being used.  

