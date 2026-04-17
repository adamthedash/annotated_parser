use std::ops::Range;

use crate::{AnnotatedResult, Annotation, AnnotationResult, ParseResult};

pub trait FoldAnnotatedResult<T, P, S>
where
    P: FnOnce() -> S,
    S: Into<String>,
{
    /// Fold the result of applying a child parser
    fn fold(
        self,
        child_annotations: Vec<Annotation>,
        offset: usize,
        parent_name: P,
        child_index: usize,
    ) -> std::result::Result<(T, usize, Vec<Annotation>), Annotation>;
}

impl<T, P, S> FoldAnnotatedResult<T, P, S> for AnnotatedResult<T>
where
    P: FnOnce() -> S,
    S: Into<String>,
{
    fn fold(
        self,
        child_annotations: Vec<Annotation>,
        offset: usize,
        parent_name: P,
        child_index: usize,
    ) -> std::result::Result<(T, usize, Vec<Annotation>), Annotation> {
        match self {
            Ok((value, annotation)) => {
                let (offset, child_annotations) =
                    fold_success(annotation, child_annotations, offset, child_index);

                Ok((value, offset, child_annotations))
            }
            Err(annotation) => {
                let annotation = fold_child_err(
                    annotation,
                    child_annotations,
                    offset,
                    parent_name(),
                    child_index,
                );

                Err(annotation)
            }
        }
    }
}

pub trait FoldParseResult<T, P, S>
where
    P: FnOnce() -> S,
    S: Into<String>,
{
    /// Fold the result of applying a child parser
    fn fold(
        self,
        offset: usize,
        parent_name: P,
        child_index: usize,
    ) -> std::result::Result<(T, usize), Annotation>;
}

impl<T, P, S> FoldParseResult<T, P, S> for ParseResult<T>
where
    P: FnOnce() -> S,
    S: Into<String>,
{
    #[inline(always)]
    fn fold(
        self,
        offset: usize,
        parent_name: P,
        child_index: usize,
    ) -> std::result::Result<(T, usize), Annotation> {
        match self {
            Ok((value, inner_offset)) => Ok((value, offset + inner_offset)),
            Err(annotation) => {
                let annotation =
                    fold_child_err(annotation, vec![], offset, parent_name(), child_index);

                Err(annotation)
            }
        }
    }
}

/// Ok path of crate::Result<T>::fold
#[inline(always)]
pub fn fold_success(
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
#[inline(always)]
pub fn fold_child_err(
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
