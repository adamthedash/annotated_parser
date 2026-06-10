# Documentation Guidelines

## General Principle

Every public item (`pub`) gets a `///` doc comment unless explicitly excluded.
Documentation should be concise but informative — one sentence for simple items,
a short paragraph for complex ones. Cross-reference related items with `[]`
links.

## What Gets Documented

### Traits
Explain the trait's purpose, who implements it, and why a user would care.
Include a `# Example` block on the trait itself if it is a primary user-facing
API (e.g. `ByteParser`). Document all associated types and constants.

### Structs
- **Description**: 2–3 sentences of what the parser does, what it consumes, and
  when it fails.
- **Example**: One minimal usage example in a `/// ```rust` (or `/// ```ignore`
  for nightly-only code) block showing how to call the parser.
- **Combinator pairs**: For combinators designed to work in tandem (e.g.
  `Configured` + `Configuring`), the example should show the **complete
  real-world pattern**, not an isolated example.

### Enums
Describe the enum's purpose and document each variant with its meaning.

### Type Aliases
Describe what the alias represents and when it is used. Note if it is
public-facing for custom parser authors (e.g. `ParseWithResult`).

### Fields and Variants
Briefly describe what the field or variant holds.

### Constants
Briefly describe the value and when it applies.

### Associated Types
Describe what the type represents in the trait's context.

### Module-Level Docs (`//!`)
Explain the module's purpose at a high level. Point to the primary entry points
for users. Mention `impl Parser` types that are not re-exported as structs (e.g.
`&[u8; N]`, `&str`). Defer to item-level docs for details. No examples at the
module level.

## What Does NOT Get Documented

- `impl` blocks (e.g. `impl Parser for Box<P>`, `impl Parser for &mut P`,
  `impl From<<Annotation> for AnnotationReturn`).
- Private helpers, macro internals, or macro-generated tuple parser
  implementations (e.g. `impl Parser for (A, B, C)`).
- Module re-export lines (cargo doc handles this automatically).

## Documentation Style by Category

### Trait Methods
For methods that are primarily convenience wrappers (e.g. `ParserAdapter`
methods), use a one-liner + `See [Struct] for more info.` pattern.

### Core Traits
For foundational traits (e.g. `Parser`), provide a comprehensive trait-level doc
explaining the trait's role, key concepts, and entry points. Document each
method's behavior and any default implementation.

### Parser Structs
Follow the **Structs** section above. Use public re-exports in examples.

### Internal-But-Public Types
Types like `AnnotationReturn` and `ParseWithResult` are documented as
public-facing API for custom parser authors, not just as internal types.

## Doc Test Conventions

- Use `/// ```rust` blocks (or `/// ```ignore` for nightly-only code).
- Always import via `annotated_parser::prelude::*` and public re-exports.
- For macro-generated parsers, use `#[doc = concat!(...)]` with `stringify!`:

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

## Cross-References

Use `[]` for intra-doc links (e.g. `[Parser::parse]`, `[ByteParser]`,
`[AnnotationMode]`). Reference related structs from trait methods and vice
versa.
