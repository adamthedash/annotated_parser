use std::fmt::{Display, Write};

/// A representation of the entire parser that is applied to each file
/// Does not hold any state
#[derive(Debug, PartialEq, Eq)]
pub struct ParserSpec {
    pub name: String,
    pub inner: Vec<ParserSpec>,
    pub friendly_name: Option<String>,
}

impl ParserSpec {
    pub fn new(name: impl Into<String>, children: Vec<ParserSpec>) -> Self {
        Self {
            name: name.into(),
            inner: children,
            friendly_name: None,
        }
    }

    /// Parser with no children
    pub fn empty(name: impl Into<String>) -> Self {
        Self::new(name, vec![])
    }

    pub fn with_friendly(self, name: impl Into<String>) -> Self {
        Self {
            friendly_name: Some(name.into()),
            ..self
        }
    }

    /// Create unique paths to each hierarchy leaf
    pub fn identifiers(&self) -> Vec<String> {
        let me = std::iter::once(self.name.clone());
        let children = self.inner.iter().enumerate().flat_map(|(i, child)| {
            child
                .identifiers()
                .into_iter()
                .map(move |suffix| format!("{}[{i}]/{}", self.name, suffix))
        });

        me.chain(children).collect()
    }
}

/// Tree-like display of spec
fn display_spec(
    spec: &ParserSpec,
    f: &mut std::fmt::Formatter<'_>,
    depth: usize,
) -> std::fmt::Result {
    for i in 0..depth {
        if i % 2 == 0 {
            f.write_char('|')?;
        } else {
            f.write_char(' ')?;
        }
    }
    f.write_str(&spec.name)?;
    if let Some(friendly) = &spec.friendly_name {
        write!(f, " @ {}", friendly)?;
    }
    f.write_char('\n')?;

    for child in spec.inner.iter() {
        display_spec(child, f, depth + 1)?;
    }

    Ok(())
}

impl Display for ParserSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_spec(self, f, 0)
    }
}
