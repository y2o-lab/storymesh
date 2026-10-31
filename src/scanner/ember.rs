use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use super::{ScanError, detector};

/// Components in app/components: Glimmer files or colocated templates.
pub(super) fn is_component(
    root: &Path,
    path: &Path,
    files: &BTreeSet<PathBuf>,
) -> Result<bool, ScanError> {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return Ok(false);
    };
    if !matches!(extension, "gjs" | "gts" | "hbs") {
        return Ok(false);
    }
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return Ok(false);
    };
    if stem.is_empty() || detector::has_excluded_suffix(stem) {
        return Ok(false);
    }
    if component_relative(root, path)?.is_none() {
        return Ok(false);
    }
    let absolute = root.join(path);
    let sources = ["gjs", "gts", "hbs"]
        .into_iter()
        .filter(|ext| files.contains(&path.with_extension(ext)))
        .count();
    if sources > 1 {
        return Err(ScanError::invalid_component(
            &absolute,
            "multiple component templates share this name; resolve the ambiguity or exclude files",
        ));
    }
    if extension == "hbs"
        && files.contains(&path.with_extension("js"))
        && files.contains(&path.with_extension("ts"))
    {
        return Err(ScanError::invalid_component(
            &absolute,
            "both JavaScript and TypeScript component classes exist",
        ));
    }
    Ok(true)
}

/// Resolve a path under app/components, including when the scan root is nested.
fn component_relative(root: &Path, path: &Path) -> Result<Option<PathBuf>, ScanError> {
    let canonical_root =
        fs::canonicalize(root).map_err(|error| ScanError::io("resolve scan root", root, error))?;
    let full = canonical_root.join(path);
    let parts: Vec<_> = full.components().collect();
    let Some(position) = parts
        .windows(2)
        .rposition(|pair| pair[0].as_os_str() == "app" && pair[1].as_os_str() == "components")
    else {
        return Ok(None);
    };
    Ok(Some(parts[position + 2..].iter().collect()))
}

/// Convert a supported component path to its Ember angle-bracket invocation.
pub(super) fn invocation(root: &Path, path: &Path) -> Result<Option<String>, ScanError> {
    let Some(relative) = component_relative(root, path)? else {
        return Ok(None);
    };
    let mut without_extension = relative;
    without_extension.set_extension("");
    let mut names = Vec::new();
    for part in without_extension.iter() {
        let Some(value) = part.to_str() else {
            return Ok(None);
        };
        if value == "index" || value.is_empty() {
            return Ok(None);
        }
        let mut word = String::new();
        for segment in value.split('-') {
            if segment.is_empty()
                || !segment
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                || !segment.as_bytes()[0].is_ascii_lowercase()
            {
                return Ok(None);
            }
            let mut chars = segment.chars();
            if let Some(first) = chars.next() {
                word.push(first.to_ascii_uppercase());
                word.extend(chars);
            }
        }
        names.push(word);
    }
    if names.is_empty() {
        return Ok(None);
    }
    Ok(Some(names.join("::")))
}
