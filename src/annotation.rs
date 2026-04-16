use std::{
    fmt::{Debug, Display},
    ops::Range,
};

#[derive(Debug)]
pub struct Annotation {
    pub parser_id: String,
    /// If this annotation is the child of another, this is the index of it within the parent
    /// parser spec
    pub child_index: Option<usize>,
    pub children: Vec<Annotation>,
    pub result: AnnotationResult,
    materialized: bool,
}

#[derive(Debug)]
pub enum AnnotationResult {
    Success {
        span: Range<usize>,
        value: Box<dyn Debug>,
    },

    /// Not enough data for the parser
    Incomplete { start: usize },

    /// Child parser has failed for any reason
    Child { start: usize },

    /// Enough data, but data was unexpected
    /// Eg. parse_digit("A")
    /// Child parsers have succeeded, but something at this level has failed
    /// Eg. Length-take of chars suceeded, but resulting string was in the expected format
    Invalid { span: Range<usize>, reason: String },
}

impl Annotation {
    #[inline(always)]
    fn new(parser_id: impl Into<String>, children: Vec<Self>, result: AnnotationResult) -> Self {
        Self {
            parser_id: parser_id.into(),
            child_index: None,
            children,
            result,
            materialized: false,
        }
    }

    #[inline(always)]
    pub fn success(
        parser_id: impl Into<String>,
        span: Range<usize>,
        value: impl std::fmt::Debug + 'static,
        children: Vec<Self>,
    ) -> Self {
        Self::new(
            parser_id,
            children,
            AnnotationResult::Success {
                span,
                value: Box::new(value),
            },
        )
    }

    #[inline(always)]
    pub fn incomplete(parser_id: impl Into<String>, start: usize, children: Vec<Self>) -> Self {
        Self::new(parser_id, children, AnnotationResult::Incomplete { start })
    }

    #[inline(always)]
    pub fn child(parser_id: impl Into<String>, start: usize, children: Vec<Self>) -> Self {
        Self::new(parser_id, children, AnnotationResult::Child { start })
    }

    #[inline(always)]
    pub fn invalid(
        parser_id: impl Into<String>,
        span: Range<usize>,
        reason: String,
        children: Vec<Self>,
    ) -> Self {
        Self::new(
            parser_id,
            children,
            AnnotationResult::Invalid { span, reason },
        )
    }

    /// If this annotation is a failure, find the source node
    pub fn err_source(&self) -> Option<&Self> {
        match self.result {
            AnnotationResult::Success { .. } => None,
            AnnotationResult::Incomplete { .. } | AnnotationResult::Invalid { .. } => Some(self),
            AnnotationResult::Child { .. } => {
                self.children.iter().flat_map(|c| c.err_source()).next()
            }
        }
    }

    pub fn max_depth(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|c| c.max_depth())
            .max()
            .unwrap_or(0)
    }

    /// Helper function which updates child annotations with information from the parent parser
    fn update_with_parent(&mut self, mut offset: usize, prefix: &str) {
        // Update this parser
        // PERF: insert over format & re-assign to avoid lots of reallocs
        if let Some(index) = self.child_index {
            // format!("{prefix}[{index}]/{}", self.parser_id)
            let index = index.to_string();
            self.parser_id.reserve(prefix.len() + index.len() + 3);
            self.parser_id.insert_str(0, "]/");
            self.parser_id.insert_str(0, &index);
            self.parser_id.insert(0, '[');
        } else {
            // format!("{prefix}/{}", self.parser_id)
            self.parser_id.reserve(prefix.len() + 1);
            self.parser_id.insert(0, '/');
        };
        self.parser_id.insert_str(0, prefix);

        self.result.shift_span(offset);
        (offset, _) = self.result.span();

        // Update children
        for child in &mut self.children {
            child.update_with_parent(offset, &self.parser_id);
        }

        self.materialized = true;
    }

    /// Recursively updates all annotations in this tree, adjusting their span/offset and
    /// materializing the full paths for parser IDs.
    pub fn materialize(&mut self) {
        assert!(
            !self.materialized,
            "Annotations can only be materialised once!"
        );

        // Update children
        for child in &mut self.children {
            child.update_with_parent(0, &self.parser_id);
        }

        self.materialized = true;
    }

    /// Recurse through the annotation tree and find the first instance that matches the given
    /// parser
    pub fn find_annotation(&self, parser_id: &str) -> Option<&Annotation> {
        if self.parser_id == parser_id {
            return Some(self);
        }

        if !parser_id.starts_with(&self.parser_id) {
            // Child parsers will start with this one as a prefix
            return None;
        }

        self.children
            .iter()
            .flat_map(|c| c.find_annotation(parser_id))
            .next()
    }
}

impl AnnotationResult {
    #[inline(always)]
    pub fn span(&self) -> (usize, Option<usize>) {
        use AnnotationResult::*;
        match self {
            Success { span, .. } | Invalid { span, .. } => (span.start, Some(span.end)),
            Incomplete { start } | Child { start } => (*start, None),
        }
    }

    #[inline(always)]
    pub fn is_ok(&self) -> bool {
        matches!(self, AnnotationResult::Success { .. })
    }

    /// Shift the span/offset for this annotation forward
    #[inline(always)]
    pub fn shift_span(&mut self, offset: usize) {
        use AnnotationResult::*;
        match self {
            Success { span, .. } | Invalid { span, .. } => {
                span.start += offset;
                span.end += offset;
            }
            Incomplete { start } | Child { start } => *start += offset,
        }
    }
}

impl Display for AnnotationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AnnotationResult::*;
        match self {
            Success { value, .. } => write!(f, "{:?}", value),
            Incomplete { .. } => f.write_str("ERR(INCOMPLETE)"),
            Child { .. } => f.write_str("ERR(CHILD)"),
            Invalid { reason, .. } => write!(f, "ERR({reason})"),
        }
    }
}
