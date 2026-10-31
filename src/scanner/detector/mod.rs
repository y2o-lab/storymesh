mod angular;
mod react;
mod svelte;
mod vue;
mod web_components;

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use crate::{domain::Framework, scanner::ScanError};

pub(super) fn is_component(
    root: &Path,
    path: &Path,
    framework: Framework,
    files: &BTreeSet<PathBuf>,
) -> Result<bool, ScanError> {
    match framework {
        Framework::React | Framework::Preact | Framework::Solid | Framework::Qwik => {
            Ok(react::is_component(path))
        }
        Framework::Vue => Ok(vue::is_component(path)),
        Framework::Svelte => Ok(svelte::is_component(path)),
        Framework::Angular => angular::is_component(root, path),
        Framework::WebComponents => web_components::is_component(root, path),
        Framework::Html => Ok(path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".component.html"))
            .is_some_and(|stem| !stem.is_empty() && !has_excluded_suffix(stem))),
        Framework::Ember => super::ember::is_component(root, path, files),
    }
}

fn has_declaration_extension(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

pub(super) fn has_excluded_suffix(stem: &str) -> bool {
    [".test", ".spec", ".stories", ".story"]
        .iter()
        .any(|suffix| stem.ends_with(suffix))
}

fn is_component_index(path: &Path) -> bool {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .is_some_and(is_pascal_case)
}

fn is_pascal_case(stem: &str) -> bool {
    stem.chars().next().is_some_and(char::is_uppercase)
}
