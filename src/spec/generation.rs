use super::{
    ComponentSpec, IndexedStory, SpecCheckError, StoryRequirement,
    index::read_index,
    matching::normalize_story_name,
    parser::{Component, read_existing_specs, valid_id},
};
use crate::{
    Framework, ScanOptions, scan_with_options,
    scanner::{component_name, component_title, story_skeleton},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

/// Files created or skipped by a generation command.
#[derive(Debug, Default)]
pub struct GenerationResult {
    /// Paths created during this invocation, including story skeletons.
    pub created: Vec<PathBuf>,
    /// Number of newly created specification files.
    pub created_specs: usize,
    /// Existing specification paths updated by an explicit merge.
    pub updated: Vec<PathBuf>,
    /// Number of already managed components or titles skipped.
    pub skipped: usize,
    /// Drafts created for components that already have a story file.
    pub existing_story_drafts: usize,
    /// Story skeletons created by `init --with-stories`.
    pub generated_stories: usize,
}

struct PlannedFile {
    path: PathBuf,
    contents: String,
}

struct PlannedUpdate {
    path: PathBuf,
    contents: String,
}

/// Create YAML drafts from scanned components, optionally with story skeletons.
pub fn init(
    root: &Path,
    framework: Framework,
    spec_dir: &Path,
    with_stories: bool,
) -> Result<GenerationResult, SpecCheckError> {
    let report = scan_with_options(root, framework, &ScanOptions::default())
        .map_err(|e| SpecCheckError(e.to_string()))?;
    if report.components.is_empty() {
        return Err(SpecCheckError(format!(
            "{}: no components found",
            root.display()
        )));
    }
    let existing = read_existing_specs(spec_dir)?;
    let mut ids = existing
        .iter()
        .map(|s| (s.component.id.clone(), s.source_path.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut planned = Vec::new();
    let mut created_specs = 0;
    let mut skipped = 0;
    let mut existing_story_drafts = 0;
    let mut generated_stories = 0;
    let mut generated_titles = BTreeSet::new();
    for component in &report.components {
        let relative = &component.component;
        let stem_path = relative.with_extension("");
        let spec_path = spec_dir.join(relative).with_extension("yaml");
        if spec_path.is_file() {
            skipped += 1;
            continue;
        }
        if spec_path.exists() {
            return Err(SpecCheckError(format!(
                "{}: target is not a regular file",
                spec_path.display()
            )));
        }
        let stem = relative
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                SpecCheckError(format!("{}: invalid component name", relative.display()))
            })?;
        let generated_title = component_title(relative, stem);
        let name = generated_title
            .strip_prefix("Components/")
            .unwrap_or(component_name(relative))
            .to_owned();
        let id = normalize_path_id(&stem_path)?;
        if let Some(previous) = ids.insert(id.clone(), spec_path.clone()) {
            return Err(SpecCheckError(format!(
                "{}: component id {id} collides with {}",
                spec_path.display(),
                previous.display()
            )));
        }
        let mut spec = ComponentSpec {
            version: 1,
            component: Component {
                id,
                name: name.clone(),
                title: None,
            },
            stories: Vec::new(),
            source_path: PathBuf::new(),
        };
        if with_stories && component.story.is_none() {
            let title = generated_title;
            if existing
                .iter()
                .any(|s| s.component.title.as_deref() == Some(&title))
            {
                return Err(SpecCheckError(format!(
                    "{}: generated title {title} already belongs to a spec",
                    relative.display()
                )));
            }
            if !generated_titles.insert(title.clone()) {
                return Err(SpecCheckError(format!(
                    "{}: generated title {title} is ambiguous",
                    relative.display()
                )));
            }
            let (story_path, contents) = story_skeleton(root, relative, framework)
                .map_err(|e| SpecCheckError(e.to_string()))?;
            let absolute = root.join(story_path);
            reject_symlink_under(root, &absolute)?;
            if absolute.exists() {
                return Err(SpecCheckError(format!(
                    "{}: target already exists",
                    absolute.display()
                )));
            }
            spec.component.title = Some(title);
            spec.stories.push(StoryRequirement {
                id: "default".into(),
                required: true,
            });
            planned.push(PlannedFile {
                path: absolute,
                contents,
            });
            generated_stories += 1;
        } else if component.story.is_some() {
            existing_story_drafts += 1;
        }
        let mut yaml = serde_yaml::to_string(&spec).map_err(|e| SpecCheckError(e.to_string()))?;
        if spec.stories.is_empty() {
            yaml = yaml.replace(
                "stories: []\n",
                "stories: [] # TODO: list required UI states\n",
            );
        }
        reject_symlink_under(spec_dir, &spec_path)?;
        created_specs += 1;
        planned.push(PlannedFile {
            path: spec_path,
            contents: yaml,
        });
    }
    let created = create_files(planned)?;
    Ok(GenerationResult {
        created,
        created_specs,
        updated: Vec::new(),
        skipped,
        existing_story_drafts,
        generated_stories,
    })
}

/// Create YAML specs from indexed stories, optionally merging new entries.
pub fn import(
    index_path: &Path,
    spec_dir: &Path,
    merge: bool,
) -> Result<GenerationResult, SpecCheckError> {
    let stories = read_index(index_path)?;
    let existing = read_existing_specs(spec_dir)?;
    let mut groups: BTreeMap<&str, Vec<&IndexedStory>> = BTreeMap::new();
    for story in &stories {
        groups.entry(&story.title).or_default().push(story);
    }
    let mut planned = Vec::new();
    let mut updates = Vec::new();
    let mut skipped = 0;
    let mut ids = existing
        .iter()
        .map(|s| (s.component.id.clone(), s.source_path.clone()))
        .collect::<BTreeMap<_, _>>();
    for (title, group) in groups {
        let representative = group[0];
        let name = title.rsplit('/').next().unwrap_or(title);
        let mut requirements = Vec::new();
        let mut story_ids = BTreeSet::new();
        for story in group {
            let story_id = normalize_story_name(&story.name);
            if !valid_id(&story_id) || !story_ids.insert(story_id.clone()) {
                return Err(SpecCheckError(format!(
                    "{}: story {} title '{}' name '{}' has invalid or duplicate normalized id '{}'",
                    index_path.display(),
                    story.id,
                    title,
                    story.name,
                    story_id
                )));
            }
            requirements.push(StoryRequirement {
                id: story_id,
                required: true,
            });
        }
        requirements.sort_by(|a, b| a.id.cmp(&b.id));
        if let Some(spec) = existing
            .iter()
            .find(|s| s.component.title.as_deref() == Some(title))
        {
            if merge {
                let declared = spec.stories.iter().map(|s| &s.id).collect::<BTreeSet<_>>();
                let additions = requirements
                    .iter()
                    .filter(|s| !declared.contains(&s.id))
                    .collect::<Vec<_>>();
                if additions.is_empty() {
                    skipped += 1;
                } else {
                    updates.push(PlannedUpdate {
                        path: spec.source_path.clone(),
                        contents: append_stories(&spec.source_path, &additions)?,
                    });
                }
            } else {
                skipped += 1;
            }
            continue;
        }
        if let Some(spec) = existing
            .iter()
            .find(|s| s.component.title.is_none() && s.component.name == name)
        {
            return Err(SpecCheckError(format!(
                "{}: title {title} may match this spec; set component.title",
                spec.source_path.display()
            )));
        }
        let id = normalize_title_id(title).map_err(|error| {
            SpecCheckError(format!(
                "{}: story {} title '{}' name '{}': {error}",
                index_path.display(),
                representative.id,
                title,
                representative.name
            ))
        })?;
        let path = spec_dir.join(format!("{id}.yaml"));
        reject_symlink_under(spec_dir, &path)?;
        if let Some(previous) = ids.insert(id.clone(), path.clone()) {
            return Err(SpecCheckError(format!(
                "{}: story {} title '{}' name '{}': component id {id} for {} collides with {}",
                index_path.display(),
                representative.id,
                title,
                representative.name,
                path.display(),
                previous.display()
            )));
        }
        if path.exists() {
            return Err(SpecCheckError(format!(
                "{}: story {} title '{}' name '{}': target {} already exists",
                index_path.display(),
                representative.id,
                title,
                representative.name,
                path.display()
            )));
        }
        let spec = ComponentSpec {
            version: 1,
            component: Component {
                id,
                name: name.to_owned(),
                title: Some(title.to_owned()),
            },
            stories: requirements,
            source_path: PathBuf::new(),
        };
        let yaml = serde_yaml::to_string(&spec).map_err(|e| SpecCheckError(e.to_string()))?;
        planned.push(PlannedFile {
            path,
            contents: yaml,
        });
    }
    let created_specs = planned.len();
    let created = create_files(planned)?;
    let mut updated = Vec::new();
    for update in updates {
        if let Err(error) = write_update(&update) {
            return Err(SpecCheckError(format!(
                "{error}; newly created files: {}",
                created
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        updated.push(update.path);
    }
    Ok(GenerationResult {
        created,
        created_specs,
        updated,
        skipped,
        existing_story_drafts: 0,
        generated_stories: 0,
    })
}

fn append_stories(path: &Path, additions: &[&StoryRequirement]) -> Result<String, SpecCheckError> {
    let source =
        fs::read_to_string(path).map_err(|e| SpecCheckError(format!("{}: {e}", path.display())))?;
    let mut in_stories = false;
    for line in source.lines() {
        let trim = line.trim();
        if trim.starts_with("---")
            || trim.starts_with("...")
            || trim.contains('&')
            || trim.contains('*')
            || trim.starts_with('[')
            || trim.starts_with('{')
        {
            return Err(SpecCheckError(format!(
                "{}: --merge supports only a simple v1 block list without anchors or flow syntax",
                path.display()
            )));
        }
        if line == "stories:" {
            in_stories = true;
            continue;
        }
        if in_stories
            && !trim.is_empty()
            && !trim.starts_with('#')
            && !line.starts_with("  - id:")
            && !line.starts_with("    required:")
        {
            return Err(SpecCheckError(format!(
                "{}: --merge supports only a simple stories block list at the end of the file",
                path.display()
            )));
        }
    }
    if !in_stories {
        return Err(SpecCheckError(format!(
            "{}: --merge requires a block-form stories list",
            path.display()
        )));
    }
    let mut output = source;
    if !output.ends_with('\n') {
        output.push('\n');
    }
    for story in additions {
        output.push_str(&format!("  - id: {}\n    required: true\n", story.id));
    }
    Ok(output)
}

fn write_update(update: &PlannedUpdate) -> Result<(), SpecCheckError> {
    let path = &update.path;
    let name = path
        .file_name()
        .ok_or_else(|| SpecCheckError(format!("{}: invalid file name", path.display())))?
        .to_string_lossy();
    let parent = path
        .parent()
        .ok_or_else(|| SpecCheckError(format!("{}: no parent", path.display())))?;
    let backup = parent.join(format!(".{name}.storymesh-backup-{}", std::process::id()));
    let temp = parent.join(format!(".{name}.storymesh-temp-{}", std::process::id()));
    for candidate in [&backup, &temp] {
        if candidate.exists() {
            return Err(SpecCheckError(format!(
                "{}: temporary path already exists",
                candidate.display()
            )));
        }
    }
    let mut source = fs::File::open(path)
        .map_err(|e| SpecCheckError(format!("{}: backup read failed: {e}", path.display())))?;
    let mut backup_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup)
        .map_err(|e| {
            SpecCheckError(format!("{}: backup creation failed: {e}", backup.display()))
        })?;
    std::io::copy(&mut source, &mut backup_file)
        .map_err(|e| SpecCheckError(format!("{}: backup write failed: {e}", backup.display())))?;
    backup_file
        .sync_all()
        .map_err(|e| SpecCheckError(format!("{}: backup sync failed: {e}", backup.display())))?;
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| SpecCheckError(format!("{}: {e}", temp.display())))?;
        file.write_all(update.contents.as_bytes())
            .map_err(|e| SpecCheckError(format!("{}: {e}", temp.display())))?;
        let permissions = fs::metadata(path)
            .map_err(|e| SpecCheckError(format!("{}: {e}", path.display())))?
            .permissions();
        fs::set_permissions(&temp, permissions)
            .map_err(|e| SpecCheckError(format!("{}: {e}", temp.display())))?;
        file.sync_all()
            .map_err(|e| SpecCheckError(format!("{}: {e}", temp.display())))?;
        fs::rename(&temp, path).map_err(|e| {
            SpecCheckError(format!(
                "{}: atomic replacement failed: {e}",
                path.display()
            ))
        })
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temp);
        return Err(SpecCheckError(format!(
            "{error}; original backup: {}",
            backup.display()
        )));
    }
    fs::remove_file(&backup).map_err(|e| {
        SpecCheckError(format!(
            "{}: update succeeded but backup cleanup failed: {e}",
            backup.display()
        ))
    })?;
    Ok(())
}

fn normalize_title_id(title: &str) -> Result<String, SpecCheckError> {
    let segments = title
        .split('/')
        .map(normalize_story_name)
        .collect::<Vec<_>>();
    if segments.iter().any(|s| !valid_id(s)) {
        return Err(SpecCheckError(format!(
            "title '{title}' cannot be represented as a v1 id"
        )));
    }
    Ok(segments.join("-"))
}
fn normalize_path_id(path: &Path) -> Result<String, SpecCheckError> {
    let parts = path
        .iter()
        .map(|s| normalize_story_name(&s.to_string_lossy().replace('.', "-")))
        .collect::<Vec<_>>();
    if parts.iter().any(|s| !valid_id(s)) {
        return Err(SpecCheckError(format!(
            "{}: cannot be represented as a v1 id",
            path.display()
        )));
    }
    Ok(parts.join("-"))
}

fn reject_symlink_under(root: &Path, target: &Path) -> Result<(), SpecCheckError> {
    if fs::symlink_metadata(root).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(SpecCheckError(format!(
            "{}: output root must not be a symlink",
            root.display()
        )));
    }
    let relative = target.strip_prefix(root).map_err(|_| {
        SpecCheckError(format!(
            "{}: target escapes output root {}",
            target.display(),
            root.display()
        ))
    })?;
    let mut path = root.to_path_buf();
    for part in relative.components() {
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(SpecCheckError(format!(
                    "{}: output path must not contain a symlink",
                    path.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(SpecCheckError(format!("{}: {error}", path.display()))),
        }
    }
    Ok(())
}

fn create_files(planned: Vec<PlannedFile>) -> Result<Vec<PathBuf>, SpecCheckError> {
    let mut paths = BTreeSet::new();
    for file in &planned {
        if !paths.insert(file.path.clone()) || file.path.exists() {
            return Err(SpecCheckError(format!(
                "{}: target already exists or collides",
                file.path.display()
            )));
        }
        let mut ancestor = file.path.parent();
        while let Some(path) = ancestor {
            if path.exists() && !path.is_dir() {
                return Err(SpecCheckError(format!(
                    "{}: parent is not a directory",
                    path.display()
                )));
            }
            ancestor = path.parent();
        }
    }
    let mut created = Vec::new();
    for file in planned {
        let result = (|| {
            let parent = file
                .path
                .parent()
                .ok_or_else(|| SpecCheckError(format!("{}: has no parent", file.path.display())))?;
            fs::create_dir_all(parent)
                .map_err(|e| SpecCheckError(format!("{}: {e}", parent.display())))?;
            let mut out = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&file.path)
                .map_err(|e| SpecCheckError(format!("{}: {e}", file.path.display())))?;
            created.push(file.path.clone());
            out.write_all(file.contents.as_bytes())
                .map_err(|e| SpecCheckError(format!("{}: {e}", file.path.display())))
        })();
        if let Err(error) = result {
            let mut left = Vec::new();
            for path in &created {
                if fs::remove_file(path).is_err() {
                    left.push(path.display().to_string());
                }
            }
            return Err(SpecCheckError(if left.is_empty() {
                error.to_string()
            } else {
                format!(
                    "{error}; rollback failed, created paths: {}",
                    left.join(", ")
                )
            }));
        }
    }
    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::{PlannedFile, create_files};
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn failed_later_write_removes_only_new_files() {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("storymesh-rollback-{}-{id}", std::process::id()));
        fs::create_dir(&root).expect("test directory is created");
        let existing = root.join("keep.txt");
        fs::write(&existing, "keep").expect("existing file is created");
        let first = root.join("new-file");
        let second = first.join("child");
        let result = create_files(vec![
            PlannedFile {
                path: first.clone(),
                contents: "new".into(),
            },
            PlannedFile {
                path: second,
                contents: "cannot be created".into(),
            },
        ]);
        assert!(result.is_err());
        assert!(
            !first.exists(),
            "the first newly created file is rolled back"
        );
        assert_eq!(fs::read_to_string(existing).unwrap(), "keep");
        fs::remove_dir_all(root).expect("test directory is removed");
    }
}
