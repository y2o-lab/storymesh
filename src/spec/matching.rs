use super::{ComponentSpec, IndexedStory, SpecCheckError, SpecCheckResult, SpecIssue};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Normalize a Storybook display name to a spec story ID.
pub fn normalize_story_name(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            if !out.is_empty() && !out.ends_with('-') {
                out.push('-');
            }
        } else {
            out.push(c.to_ascii_lowercase());
        }
    }
    out.trim_end_matches('-').to_owned()
}

pub fn check(
    specs: &[ComponentSpec],
    stories: &[IndexedStory],
    index_path: &Path,
) -> Result<SpecCheckResult, SpecCheckError> {
    let mut titles: BTreeMap<&str, BTreeMap<String, &IndexedStory>> = BTreeMap::new();
    for story in stories {
        let name = normalize_story_name(&story.name);
        let group = titles.entry(&story.title).or_default();
        if let Some(previous) = group.insert(name.clone(), story) {
            return Err(SpecCheckError(format!(
                "{}: {} and {}: title '{}' has ambiguous story name '{}'",
                index_path.display(),
                previous.id,
                story.id,
                story.title,
                name
            )));
        }
    }
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut stories_checked = 0;
    let mut resolved_titles = BTreeMap::new();
    let mut display_names = BTreeMap::new();
    for spec in specs {
        *display_names
            .entry(spec.component.name.as_str())
            .or_insert(0usize) += 1;
    }
    for spec in specs {
        let title = if let Some(explicit) = &spec.component.title {
            explicit.as_str()
        } else {
            let candidates = titles
                .keys()
                .filter(|title| {
                    **title == spec.component.name
                        || title.rsplit('/').next() == Some(spec.component.name.as_str())
                })
                .copied()
                .collect::<Vec<_>>();
            if candidates.len() > 1 {
                return Err(SpecCheckError(format!(
                    "{}: '{}' matches multiple titles; set component.title",
                    spec.source_path.display(),
                    spec.component.name
                )));
            }
            candidates.first().copied().unwrap_or(&spec.component.name)
        };
        if let Some(previous) = resolved_titles.insert(title.to_owned(), &spec.source_path) {
            return Err(SpecCheckError(format!(
                "{}: title '{title}' also resolves from {}; set distinct component.title values",
                spec.source_path.display(),
                previous.display()
            )));
        }
        let found = titles.get(title);
        stories_checked += found.map_or(0, |v| v.len());
        let label = if display_names[spec.component.name.as_str()] > 1 {
            format!("{} ({})", spec.component.name, spec.component.id)
        } else {
            spec.component.name.clone()
        };
        let declared = spec
            .stories
            .iter()
            .map(|s| s.id.as_str())
            .collect::<BTreeSet<_>>();
        for requirement in &spec.stories {
            if requirement.required
                && !found.is_some_and(|group| group.contains_key(&requirement.id))
            {
                errors.push(SpecIssue::MissingStory {
                    component: label.clone(),
                    story: requirement.id.clone(),
                });
            }
        }
        if let Some(group) = found {
            for name in group.keys() {
                if !declared.contains(name.as_str()) {
                    warnings.push(SpecIssue::UndeclaredStory {
                        component: label.clone(),
                        story: name.clone(),
                    });
                }
            }
        }
    }
    let sort_key = |issue: &SpecIssue| match issue {
        SpecIssue::MissingStory { component, story }
        | SpecIssue::UndeclaredStory { component, story } => (component.clone(), story.clone()),
    };
    errors.sort_by_key(&sort_key);
    warnings.sort_by_key(&sort_key);
    Ok(SpecCheckResult {
        valid: errors.is_empty(),
        components_checked: specs.len(),
        stories_checked,
        errors,
        warnings,
    })
}
