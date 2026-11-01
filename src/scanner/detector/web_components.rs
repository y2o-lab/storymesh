use std::{fs, path::Path};

use super::{has_declaration_extension, has_excluded_suffix};
use crate::scanner::{ScanError, matching, web_components};

pub(super) fn is_component(root: &Path, path: &Path) -> Result<bool, ScanError> {
    if matching::is_story_file(path) || has_declaration_extension(path) {
        return Ok(false);
    }
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return Ok(false);
    };
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return Ok(false);
    };
    if !matches!(extension.to_ascii_lowercase().as_str(), "js" | "ts") || has_excluded_suffix(stem)
    {
        return Ok(false);
    }
    let absolute = root.join(path);
    let source = fs::read(&absolute)
        .map_err(|error| ScanError::io("read Web Components source", &absolute, error))?;
    Ok(!web_components::registrations(&source).is_empty())
}
