mod generation;
mod index;
mod matching;
mod parser;

use std::{error::Error, fmt, path::Path};

pub use generation::{GenerationResult, import, init};
pub use index::IndexedStory;
pub use matching::normalize_story_name;
pub use parser::{ComponentSpec, StoryRequirement};

pub use crate::domain::{SpecCheckResult, SpecIssue};

/// Input, ambiguity, or I/O failure that prevents a trustworthy spec check.
#[derive(Debug)]
pub struct SpecCheckError(pub(crate) String);

impl fmt::Display for SpecCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl Error for SpecCheckError {}

/// Compare all YAML specifications in `spec_dir` with Storybook `index_path`.
pub fn check_spec(spec_dir: &Path, index_path: &Path) -> Result<SpecCheckResult, SpecCheckError> {
    let specs = parser::read_specs(spec_dir)?;
    let stories = index::read_index(index_path)?;
    matching::check(&specs, &stories, index_path)
}
