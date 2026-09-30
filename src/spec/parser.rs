use super::SpecCheckError;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

/// One YAML v1 component specification.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSpec {
    /// Schema version, currently `1`.
    pub version: u8,
    /// Identity and Storybook title of the component.
    pub component: Component,
    /// Declared UI states.
    pub stories: Vec<StoryRequirement>,
    #[serde(skip)]
    /// Source path for diagnostics.
    pub source_path: PathBuf,
}

/// Component identity and optional exact Storybook title.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    /// Unique spec identity.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Exact Storybook title, when specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// One declared Storybook story state.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoryRequirement {
    /// Normalized story name.
    pub id: String,
    /// Whether absence is an error.
    #[serde(default = "default_required")]
    pub required: bool,
}
fn default_required() -> bool {
    true
}

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

pub fn read_specs(root: &Path) -> Result<Vec<ComponentSpec>, SpecCheckError> {
    let specs = read_existing_specs(root)?;
    for spec in &specs {
        if spec.stories.is_empty() {
            return Err(SpecCheckError(format!(
                "{}: stories: [] is a draft; list required UI states",
                spec.source_path.display()
            )));
        }
    }
    if specs.is_empty() {
        return Err(SpecCheckError(format!(
            "{}: no YAML specs found",
            root.display()
        )));
    }
    Ok(specs)
}

pub fn read_existing_specs(root: &Path) -> Result<Vec<ComponentSpec>, SpecCheckError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    if fs::symlink_metadata(root)
        .map_err(|e| SpecCheckError(format!("{}: {e}", root.display())))?
        .file_type()
        .is_symlink()
    {
        return Err(SpecCheckError(format!(
            "{}: spec directory must not be a symlink",
            root.display()
        )));
    }
    let mut paths = Vec::new();
    collect(root, &mut paths)?;
    paths.sort();
    let mut specs = Vec::new();
    let mut ids = BTreeMap::new();
    let mut titles = BTreeMap::new();
    for path in paths {
        let content = fs::read_to_string(&path)
            .map_err(|e| SpecCheckError(format!("{}: {e}", path.display())))?;
        let mut spec: ComponentSpec = serde_yaml::from_str(&content)
            .map_err(|e| SpecCheckError(format!("{}: {e}", path.display())))?;
        spec.source_path = path.clone();
        validate(&spec)?;
        if let Some(previous) = ids.insert(spec.component.id.clone(), path.clone()) {
            return Err(SpecCheckError(format!(
                "{}: component id '{}' duplicates {}",
                path.display(),
                spec.component.id,
                previous.display()
            )));
        }
        if let Some(title) = &spec.component.title {
            if let Some(previous) = titles.insert(title.clone(), path.clone()) {
                return Err(SpecCheckError(format!(
                    "{}: title '{}' duplicates {}",
                    path.display(),
                    title,
                    previous.display()
                )));
            }
        }
        specs.push(spec);
    }
    Ok(specs)
}

fn validate(spec: &ComponentSpec) -> Result<(), SpecCheckError> {
    let path = spec.source_path.display();
    if spec.version != 1 {
        return Err(SpecCheckError(format!(
            "{path}: unsupported version {}",
            spec.version
        )));
    }
    if !valid_id(&spec.component.id) {
        return Err(SpecCheckError(format!(
            "{path}: invalid component id '{}'",
            spec.component.id
        )));
    }
    if spec.component.name.trim().is_empty() {
        return Err(SpecCheckError(format!("{path}: component name is empty")));
    }
    if spec
        .component
        .title
        .as_ref()
        .is_some_and(|v| v.trim().is_empty())
    {
        return Err(SpecCheckError(format!("{path}: component title is empty")));
    }
    let mut ids = BTreeSet::new();
    for story in &spec.stories {
        if !valid_id(&story.id) {
            return Err(SpecCheckError(format!(
                "{path}: invalid story id '{}'",
                story.id
            )));
        }
        if !ids.insert(&story.id) {
            return Err(SpecCheckError(format!(
                "{path}: duplicate story id '{}'",
                story.id
            )));
        }
    }
    Ok(())
}

fn collect(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<(), SpecCheckError> {
    let entries =
        fs::read_dir(dir).map_err(|e| SpecCheckError(format!("{}: {e}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| SpecCheckError(format!("{}: {e}", dir.display())))?;
        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') || name == "node_modules" {
            continue;
        }
        let ty = entry
            .file_type()
            .map_err(|e| SpecCheckError(format!("{}: {e}", path.display())))?;
        if ty.is_symlink() {
            continue;
        }
        if ty.is_dir() {
            collect(&path, paths)?;
        } else if ty.is_file()
            && matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("yaml" | "yml")
            )
        {
            paths.push(path);
        }
    }
    Ok(())
}
