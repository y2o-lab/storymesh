/// A missing required story or a registered story absent from the spec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecIssue {
    /// A required story was not registered.
    MissingStory { component: String, story: String },
    /// A registered story was not declared.
    UndeclaredStory { component: String, story: String },
}

/// Result of comparing component specifications with a Storybook index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecCheckResult {
    /// Whether all required stories are registered.
    pub valid: bool,
    /// Number of loaded component specs.
    pub components_checked: usize,
    /// Number of indexed stories under resolved spec titles.
    pub stories_checked: usize,
    /// Missing required stories.
    pub errors: Vec<SpecIssue>,
    /// Registered stories absent from their spec.
    pub warnings: Vec<SpecIssue>,
}
