/// Build a parser that fills a struct's fields from a sequence of field parsers.
///
/// Each field parser is traced with its field name.
/// The resulting parser is traced with the struct name.
/// The parsers are executed in the order they are defined within the macro.
///
/// # Notes
///
/// - The struct must be [`ParserOutput`](crate::ParserOutput)
/// - Supports up to 12 fields (the tuple parser limit).
///
/// # Example
///
/// ```
/// use annotated_parser::parse_struct;
/// use annotated_parser::prelude::*;
///
/// #[derive(Debug, Clone)]
/// struct Header {
///     magic: u16,
///     count: u8,
/// }
///
/// let mut parser = parse_struct!(Header {
///     magic: u16::LE,
///     count: u8::LE,
/// });
///
/// let mut input: &[u8] = &[0x2a, 0x00, 0x03];
/// let (value, _) = parser.parse(&mut input).unwrap();
/// assert_eq!(value.magic, 0x2a);
/// assert_eq!(value.count, 3);
/// ```
#[macro_export]
macro_rules! parse_struct {
    ($struct_name:ident { $($field_name:ident: $parser: expr ),* $(,)? }) => {
        ($($parser.trace(stringify!($field_name))),*)
            .map_silent(|($($field_name),*)| $struct_name {
                $($field_name),*
            })
            .trace(stringify!($struct_name))
    };
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    #[allow(dead_code, unused_variables)]
    fn test_macro_simple() {
        #[derive(Debug, Clone)]
        struct FooBar {
            foo: u32,
            bar: u8,
        }

        let foo = u32::LE;

        let parser = parse_struct!(FooBar {
            foo: foo,
            bar: u8::LE,
        });
    }

    #[test]
    #[allow(dead_code, unused_variables)]
    fn test_macro_complex() {
        #[derive(Debug, Clone)]
        struct FooBar<G> {
            foo: G,
            bar: Vec<[f32; 4]>,
            baz: Option<u8>,
        }

        let foo = u32::LE.store();
        let foo_out = foo.output();

        let bar = f32::LE.repeat::<4>().many();

        let parser = parse_struct!(FooBar {
            foo: foo,
            bar: bar,
            baz: u8::LE.run_if(foo_out.map(|f| *f < 3))
        });
    }
}
