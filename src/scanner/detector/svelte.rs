use std::path::Path;

use super::has_excluded_suffix;

pub(super) fn is_component(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };

    extension.eq_ignore_ascii_case("svelte") && !has_excluded_suffix(stem)
}
