use std::ops::Range;

use crate::{Annotation, AnnotationResult, Result};

pub trait FoldResult<T> {
    /// Fold the result of applying a child parser
    fn fold(
        self,
        child_annotations: Vec<Annotation>,
        offset: usize,
        parent_name: &str,
        child_index: usize,
    ) -> std::result::Result<(T, usize, Vec<Annotation>), Annotation>;
}

impl<T> FoldResult<T> for Result<T> {
    fn fold(
        self,
        child_annotations: Vec<Annotation>,
        offset: usize,
        parent_name: &str,
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
                    parent_name,
                    child_index,
                );

                Err(annotation)
            }
        }
    }
}

/// Ok path of crate::Result<T>::fold
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
