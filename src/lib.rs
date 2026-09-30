mod domain;
mod scanner;
mod spec;

pub use domain::{ComponentCoverage, CoverageReport, Framework, SpecCheckResult, SpecIssue};
pub use scanner::{
    GenerateError, ScanError, ScanOptions, generate_story_skeletons, scan, scan_with_options,
};

pub use spec::{
    ComponentSpec, GenerationResult, IndexedStory, SpecCheckError, StoryRequirement, check_spec,
    import, init, normalize_story_name,
};
