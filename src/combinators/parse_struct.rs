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
