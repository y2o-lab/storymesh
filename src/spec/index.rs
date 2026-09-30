use super::SpecCheckError;
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path};

/// A registered Storybook story.
#[derive(Clone, Debug)]
pub struct IndexedStory {
    /// Storybook's entry identifier.
    pub id: String,
    /// Full sidebar title.
    pub title: String,
    /// Story display name.
    pub name: String,
}

pub fn read_index(path: &Path) -> Result<Vec<IndexedStory>, SpecCheckError> {
    let content =
        fs::read_to_string(path).map_err(|e| SpecCheckError(format!("{}: {e}", path.display())))?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| SpecCheckError(format!("{}: {e}", path.display())))?;
    let entries = json
        .get("entries")
        .and_then(Value::as_object)
        .ok_or_else(|| SpecCheckError(format!("{}: expected an entries object", path.display())))?;
    let mut stories = Vec::new();
    let mut ids = BTreeSet::new();
    for (key, entry) in entries {
        let kind = entry.get("type").and_then(Value::as_str).ok_or_else(|| {
            SpecCheckError(format!("{}: entry {key} has no type", path.display()))
        })?;
        if kind == "docs" {
            continue;
        }
        if kind != "story" {
            continue;
        }
        let field = |name: &str| -> Result<String, SpecCheckError> {
            entry
                .get(name)
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
                .map(str::to_owned)
                .ok_or_else(|| {
                    SpecCheckError(format!(
                        "{}: story entry {key} has no {name}",
                        path.display()
                    ))
                })
        };
        let id = field("id")?;
        let title = field("title")?;
        let name = field("name")?;
        if !ids.insert(id.clone()) {
            return Err(SpecCheckError(format!(
                "{}: duplicate Storybook id {id}",
                path.display()
            )));
        }
        stories.push(IndexedStory { id, title, name });
    }
    if stories.is_empty() {
        return Err(SpecCheckError(format!(
            "{}: no story entries found",
            path.display()
        )));
    }
    Ok(stories)
}
