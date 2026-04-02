use std::ops::Range;

use crate::{Annotation, AnnotationResult, Result, SpeedyResult};

pub trait FoldResult<T> {
    /// Fold the result of applying a child parser
    fn fold(
        self,
        child_annotations: Vec<Annotation>,
        offset: usize,
        parent_name: &str,
        child_index: usize,
    ) -> std::result::Result<(T, Range<usize>, Vec<Annotation>), Annotation>;
}

impl<T> FoldResult<T> for Result<T> {
    fn fold(
        self,
        child_annotations: Vec<Annotation>,
        offset: usize,
        parent_name: &str,
        child_index: usize,
    ) -> std::result::Result<(T, Range<usize>, Vec<Annotation>), Annotation> {
        match self {
            Ok((value, annotation)) => {
                let (span, child_annotations) = fold_success(
                    annotation,
                    child_annotations,
                    offset,
                    parent_name,
                    child_index,
                );

                Ok((value, span, child_annotations))
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

pub trait FoldSpeedyResult<T> {
    /// Fold the result of applying a child parser
    fn fold(self, offset: usize, parent_name: &str, child_index: usize) -> SpeedyResult<T>;
}

impl<T> FoldSpeedyResult<T> for SpeedyResult<T> {
    fn fold(self, offset: usize, parent_name: &str, child_index: usize) -> SpeedyResult<T> {
        self.map_err(|a| fold_child_err(a, vec![], offset, parent_name, child_index))
    }
}

/// Ok path of crate::Result<T>::fold
pub fn fold_success(
    mut annotation: Annotation,
    mut child_annotations: Vec<Annotation>,
    offset: usize,
    parent_name: &str,
    child_index: usize,
) -> (Range<usize>, Vec<Annotation>) {
    let prefix = format!("{parent_name}[{child_index}]/");
    annotation.update_with_parent(offset, &prefix);

    let AnnotationResult::Success { span, .. } = &annotation.result else {
        unreachable!("Child parser has succeeded");
    };
    let span = span.clone();

    child_annotations.push(annotation);
    (span, child_annotations)
}

/// Error path of crate::Result<T>::fold
pub fn fold_child_err(
    mut annotation: Annotation,
    mut child_annotations: Vec<Annotation>,
    offset: usize,
    parent_name: &str,
    child_index: usize,
) -> Annotation {
    let prefix = format!("{parent_name}[{child_index}]/");
    annotation.update_with_parent(offset, &prefix);
    child_annotations.push(annotation);

    Annotation::child(parent_name, 0, child_annotations)
}
