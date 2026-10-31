/// Component library whose Storybook coverage is being inspected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Framework {
    /// React components and Component Story Format files.
    React,
    /// Preact components and Component Story Format files.
    Preact,
    /// SolidJS components and Component Story Format files.
    Solid,
    /// Qwik components and Component Story Format files.
    Qwik,
    /// JavaScript or TypeScript custom elements, including Lit elements.
    WebComponents,
    /// Explicitly named HTML component fragments and Component Story Format files.
    Html,
    /// Ember components in the supported app/components layouts.
    Ember,
    /// Vue single-file components and Component Story Format files.
    Vue,
    /// Svelte single-file components and JavaScript, TypeScript, or native Svelte stories.
    Svelte,
    /// Angular components and Component Story Format files.
    Angular,
}

impl Framework {
    /// Human-readable framework name used in reports.
    pub const fn name(self) -> &'static str {
        match self {
            Self::React => "React",
            Self::Preact => "Preact",
            Self::Solid => "SolidJS",
            Self::Qwik => "Qwik",
            Self::WebComponents => "Web Components",
            Self::Html => "HTML",
            Self::Ember => "Ember",
            Self::Vue => "Vue",
            Self::Svelte => "Svelte",
            Self::Angular => "Angular",
        }
    }
}
