use std::ops::Range;

use crate::{Annotation, AnnotationMode, AnnotationResult, AnnotationReturn};

/// Accumulate the result of a child parser into the parent context.
///
/// This trait is implemented on [`ParseWithResult`](crate::parser::ParseWithResult)
/// and can be used to reduce boilerplate associated with the bookkeeping of byte
/// offsets, child annotations, and failure propagation.
///
/// When implementing a custom combinator, you call `parse_with` on each child
/// parser and then `.fold()` on the result to update the consumed offset and
/// collect child annotations (in success mode) or wrap child failures into a
/// `Child` annotation (in fail mode).
///
/// # Example
///
/// In a custom combinator, you would typically call `parse_with` on each child
/// and then `.fold()` to accumulate the results:
///
/// ```rust ignore
/// // Parser::parse_with(...) {
///     let mut child_annotations = mode.success.then(Vec::new);
///     let mut offset = 0;
///
///     // Run the inner parser and accumulate its result
///     let value;
///     (value, offset, child_annotations) = u8::LE
///         .parse_with(input, mode)
///         .fold(mode, || self.name(), child_annotations, offset, 0)?;
///
///     // Rest of parse_with body...
/// // }
/// ```
pub trait FoldParseWithResult<T, P, S>
where
    P: FnOnce() -> S,
    S: Into<String>,
{
    /// Accumulate the result of this parser into the surrounding context.
    ///
    /// Returns the parsed value, the updated byte offset, and the accumulated child
    /// annotations on success. On failure, returns an `AnnotationReturn` that is
    /// either a `Child` annotation (in fail mode) or an unannotated start offset.
    ///
    /// # Parameters
    ///
    /// - `annotation_mode`: Controls whether success and failure paths are annotated.
    /// - `parent_name`: Closure returning the parent parser's name for failure annotations.
    /// - `child_annotations`: Optional vector of already collected child annotations.
    /// - `offset`: Current byte offset into the input.
    /// - `child_index`: Index of this child within the parent parser spec.
    fn fold(
        self,
        annotation_mode: AnnotationMode,
        parent_name: P,
        child_annotations: Option<Vec<Annotation>>,
        offset: usize,
        child_index: usize,
    ) -> Result<(T, usize, Option<Vec<Annotation>>), AnnotationReturn>;
}

impl<T, P, S> FoldParseWithResult<T, P, S> for Result<(T, AnnotationReturn), AnnotationReturn>
where
    P: FnOnce() -> S,
    S: Into<String>,
{
    #[inline]
    fn fold(
        self,
        annotation_mode: AnnotationMode,
        parent_name: P,
        mut child_annotations: Option<Vec<Annotation>>,
        offset: usize,
        child_index: usize,
    ) -> Result<(T, usize, Option<Vec<Annotation>>), AnnotationReturn> {
        match self {
            Ok((value, annotation)) => {
                let new_offset = if annotation_mode.success {
                    // Accumulate into child annotations
                    let (new_offset, new_child_annotations) = fold_success(
                        annotation.annotation().expect("Annotated path"),
                        child_annotations.unwrap_or_default(),
                        offset,
                        child_index,
                    );
                    child_annotations = Some(new_child_annotations);

                    new_offset
                } else {
                    // Extract offset
                    offset + annotation.span().expect("Unannoated path").end
                };

                Ok((value, new_offset, child_annotations))
            }
            Err(annotation) => {
                let annotation = if annotation_mode.fail {
                    // Accumulate into Annotation::Child
                    fold_child_err(
                        annotation.annotation().expect("Annotated path"),
                        child_annotations.unwrap_or_default(),
                        offset,
                        parent_name(),
                        child_index,
                    )
                    .into()
                } else {
                    // Extract start offset
                    let start = annotation.start().expect("Unannoated path");

                    AnnotationReturn::Start(offset + start)
                };

                Err(annotation)
            }
        }
    }
}

/// Ok path of crate::Result<T>::fold
#[inline]
fn fold_success(
    mut annotation: Annotation,
    mut child_annotations: Vec<Annotation>,
    offset: usize,
    child_index: usize,
) -> (usize, Vec<Annotation>) {
    annotation.child_index = Some(child_index);
    annotation.result.shift_span(offset);

    let AnnotationResult::Success {
        span: Range { end, .. },
        ..
    } = annotation.result
    else {
        unreachable!("Child parser has succeeded");
    };

    child_annotations.push(annotation);
    (end, child_annotations)
}

/// Error path of crate::Result<T>::fold
#[inline]
fn fold_child_err(
    mut annotation: Annotation,
    mut child_annotations: Vec<Annotation>,
    offset: usize,
    parent_name: impl Into<String>,
    child_index: usize,
) -> Annotation {
    annotation.child_index = Some(child_index);
    annotation.result.shift_span(offset);
    child_annotations.push(annotation);

    Annotation::child(parent_name, 0, child_annotations)
}
