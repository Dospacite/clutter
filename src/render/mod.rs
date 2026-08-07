mod dart;

use std::collections::BTreeMap;

use crate::model::RecoveredProgram;
use rayon::prelude::*;

pub use dart::render_support;
pub(crate) use dart::source_visible_function;

pub fn render_libraries(program: &RecoveredProgram) -> BTreeMap<std::path::PathBuf, String> {
    let index = dart::RenderIndex::new(program);
    program
        .libraries
        .par_iter()
        .map(|library| {
            (
                library.output_path.clone(),
                dart::render_library(library, program, &index),
            )
        })
        .collect()
}
