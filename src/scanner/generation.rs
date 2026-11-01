use std::{
    error::Error,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::domain::{CoverageReport, Framework};

/// Error returned when a story skeleton cannot be generated.
#[derive(Debug)]
pub struct GenerateError {
    context: String,
    source: Option<io::Error>,
}

impl GenerateError {
    fn invalid_component(component: &Path, detail: &str) -> Self {
        Self {
            context: format!(
                "cannot generate a story for {}: {detail}",
                component.display()
            ),
            source: None,
        }
    }

    fn io(operation: &str, path: &Path, source: io::Error) -> Self {
        Self {
            context: format!("failed to {operation} {}", path.display()),
            source: Some(source),
        }
    }
}

impl fmt::Display for GenerateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.context)?;
        if let Some(source) = &self.source {
            write!(formatter, ": {source}")?;
        }
        Ok(())
    }
}

impl Error for GenerateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source as &(dyn Error + 'static))
    }
}

/// Write a minimal CSF story beside every component missing one.
///
/// Existing files are never overwritten. Returned paths are relative to `root`.
pub fn generate_story_skeletons(
    root: &Path,
    report: &CoverageReport,
) -> Result<Vec<PathBuf>, GenerateError> {
    let skeletons = report
        .missing()
        .map(|component| story_skeleton(root, component, report.framework))
        .collect::<Result<Vec<_>, _>>()?;

    for (path, _) in &skeletons {
        let absolute_path = root.join(path);
        if absolute_path.exists() {
            return Err(GenerateError::invalid_component(
                path,
                "the target story file already exists",
            ));
        }
    }

    for (path, contents) in &skeletons {
        let absolute_path = root.join(path);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&absolute_path)
            .map_err(|error| GenerateError::io("create story skeleton", &absolute_path, error))?;
        file.write_all(contents.as_bytes())
            .map_err(|error| GenerateError::io("write story skeleton", &absolute_path, error))?;
    }

    Ok(skeletons.into_iter().map(|(path, _)| path).collect())
}

pub(crate) fn story_skeleton(
    root: &Path,
    component: &Path,
    framework: Framework,
) -> Result<(PathBuf, String), GenerateError> {
    let file_name = component
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| GenerateError::invalid_component(component, "file name is not UTF-8"))?;
    let stem = component
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| GenerateError::invalid_component(component, "file stem is not UTF-8"))?;
    let extension = component
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| GenerateError::invalid_component(component, "extension is not UTF-8"))?;

    let story_extension = match framework {
        Framework::React | Framework::Preact | Framework::Solid | Framework::Qwik => extension,
        Framework::WebComponents => extension,
        Framework::Html | Framework::Ember => "js",
        Framework::Vue | Framework::Svelte | Framework::Angular => "ts",
    };
    let mut story_path = component.to_path_buf();
    story_path.set_file_name(format!("{stem}.stories.{story_extension}"));

    if matches!(framework, Framework::Html | Framework::WebComponents)
        && file_name.contains(['?', '#'])
    {
        return Err(GenerateError::invalid_component(
            component,
            "file name contains a module query or fragment delimiter; write the story manually",
        ));
    }

    if framework == Framework::Ember {
        let name = super::ember::invocation(root, component)
            .map_err(|error| GenerateError::invalid_component(component, &error.to_string()))?
            .ok_or_else(|| {
                GenerateError::invalid_component(
                    component,
                    "cannot infer a conventional Ember component name; write the story manually",
                )
            })?;
        let title = escape_javascript_string(&component_title(component, stem));
        return Ok((
            story_path,
            format!(
                "import {{ hbs }} from 'ember-cli-htmlbars';\n\nconst meta = {{ title: '{title}' }};\n\nexport default meta;\n\nexport const Default = {{\n  render: () => ({{ template: hbs`<{name} />` }}),\n}};\n"
            ),
        ));
    }

    if framework == Framework::Html {
        let import_path = escape_javascript_string(&format!("./{file_name}?raw"));
        let title = escape_javascript_string(&component_title(component, stem));
        return Ok((
            story_path,
            format!(
                "import markup from '{import_path}';\n\nconst meta = {{\n  title: '{title}',\n}};\n\nexport default meta;\n\nexport const Default = {{\n  render: () => markup,\n}};\n"
            ),
        ));
    }

    if framework == Framework::WebComponents {
        let absolute_path = root.join(component);
        let source = fs::read(&absolute_path).map_err(|error| {
            GenerateError::io("read Web Components source", &absolute_path, error)
        })?;
        let tags = super::web_components::registrations(&source);
        let [Some(tag)] = tags.as_slice() else {
            return Err(GenerateError::invalid_component(
                component,
                "expected one registration with a static custom-element tag; write the story manually",
            ));
        };
        let import_path = escape_javascript_string(&format!("./{file_name}"));
        let title = escape_javascript_string(&component_title(component, stem));
        return Ok((
            story_path,
            format!(
                "import '{import_path}';\n\nconst meta = {{\n  title: '{title}',\n  component: '{tag}',\n}};\n\nexport default meta;\n\nexport const Default = {{\n  render: () => document.createElement('{tag}'),\n}};\n"
            ),
        ));
    }

    let import_stem = match framework {
        Framework::Vue | Framework::Svelte => file_name.to_owned(),
        Framework::React
        | Framework::Preact
        | Framework::Solid
        | Framework::Qwik
        | Framework::WebComponents
        | Framework::Html
        | Framework::Ember
        | Framework::Angular => stem.to_owned(),
    };
    let import_path = escape_javascript_string(&format!("./{import_stem}"));
    let import = match framework {
        Framework::React | Framework::Preact | Framework::Solid | Framework::Qwik => {
            let import = jsx_import(root, component, &import_path, framework != Framework::React)?;
            if framework != Framework::React && import.is_none() {
                return Err(GenerateError::invalid_component(
                    component,
                    "could not identify a default export or a named export matching the component name; write the story manually",
                ));
            }
            import
        }
        Framework::Vue | Framework::Svelte => {
            Some(format!("import Component from '{import_path}';"))
        }
        Framework::Angular => {
            let absolute_path = root.join(component);
            let source = fs::read(&absolute_path).map_err(|error| {
                GenerateError::io("read Angular component", &absolute_path, error)
            })?;
            let export_name = angular_export_name(stem);
            if contains_named_export(&source, &export_name) || !contains_default_export(&source) {
                Some(format!(
                    "import {{ {export_name} as Component }} from '{import_path}';"
                ))
            } else {
                Some(format!("import Component from '{import_path}';"))
            }
        }
        Framework::WebComponents => {
            return Err(GenerateError::invalid_component(
                component,
                "cannot generate this Web Components story",
            ));
        }
        Framework::Html => {
            return Err(GenerateError::invalid_component(
                component,
                "cannot generate this HTML story",
            ));
        }
        Framework::Ember => {
            return Err(GenerateError::invalid_component(
                component,
                "cannot generate this Ember story",
            ));
        }
    };
    let title = escape_javascript_string(&component_title(component, stem));
    let contents = match import {
        Some(import) => format!(
            "{import}\n\nconst meta = {{\n  title: '{title}',\n  component: Component,\n}};\n\nexport default meta;\n\nexport const Default = {{}};\n"
        ),
        None => format!(
            "const meta = {{\n  title: '{title}',\n}};\n\nexport default meta;\n\nexport const Default = {{\n  render: () => null,\n}};\n"
        ),
    };

    Ok((story_path, contents))
}

fn jsx_import(
    root: &Path,
    component: &Path,
    import_path: &str,
    strict: bool,
) -> Result<Option<String>, GenerateError> {
    let absolute_path = root.join(component);
    let source = fs::read(&absolute_path)
        .map_err(|error| GenerateError::io("read JSX component", &absolute_path, error))?;
    if contains_default_export(&source) {
        return Ok(Some(format!("import Component from '{import_path}';")));
    }

    let component_name = component_name(component);
    let export_name = pascal_case_identifier(component_name);
    if export_name.is_empty()
        || !(if strict {
            contains_named_declaration(&source, &export_name)
                || contains_export_specifier(&javascript_tokens(&source), export_name.as_bytes())
        } else {
            contains_named_export(&source, &export_name)
        })
    {
        return Ok(None);
    }
    Ok(Some(format!(
        "import {{ {export_name} as Component }} from '{import_path}';"
    )))
}

fn contains_named_declaration(source: &[u8], export_name: &str) -> bool {
    let identifiers = javascript_identifiers(source);
    let name = export_name.as_bytes();
    identifiers.windows(3).any(|tokens| {
        tokens[0] == b"export"
            && matches!(
                tokens[1],
                b"function" | b"class" | b"const" | b"let" | b"var"
            )
            && tokens[2] == name
    }) || identifiers.windows(4).any(|tokens| {
        tokens[0] == b"export"
            && tokens[1] == b"async"
            && tokens[2] == b"function"
            && tokens[3] == name
    })
}

fn contains_default_export(source: &[u8]) -> bool {
    let tokens = javascript_tokens(source);
    tokens
        .windows(2)
        .any(|tokens| tokens[0] == b"export" && tokens[1] == b"default")
        || contains_export_specifier(&tokens, b"default")
}

fn contains_export_specifier(tokens: &[&[u8]], export_name: &[u8]) -> bool {
    tokens.iter().enumerate().any(|(index, token)| {
        if *token != b"export" {
            return false;
        }
        let remaining = &tokens[index + 1..];
        if remaining.first() != Some(&b"{".as_slice()) {
            return false;
        }
        let end = remaining.iter().position(|token| *token == b"}");
        end.is_some_and(|end| {
            remaining[1..end].split(|token| *token == b",").any(|specifier| {
                matches!(specifier, [name] if *name == export_name)
                    || matches!(specifier, [_, alias, name] if *alias == b"as" && *name == export_name)
            })
        })
    })
}

fn contains_named_export(source: &[u8], export_name: &str) -> bool {
    let identifiers = javascript_identifiers(source);
    let export_name = export_name.as_bytes();
    identifiers.windows(3).any(|tokens| {
        tokens[0] == b"export"
            && matches!(
                tokens[1],
                b"function" | b"class" | b"const" | b"let" | b"var"
            )
            && tokens[2] == export_name
    }) || identifiers.windows(4).any(|tokens| {
        tokens[0] == b"export"
            && ((tokens[1] == b"async" && tokens[2] == b"function") || tokens[2] == b"as")
            && tokens[3] == export_name
    }) || identifiers
        .windows(2)
        .any(|tokens| tokens[0] == b"export" && tokens[1] == export_name)
}

fn javascript_identifiers(source: &[u8]) -> Vec<&[u8]> {
    javascript_tokens(source)
        .into_iter()
        .filter(|token| token[0].is_ascii_alphabetic() || matches!(token[0], b'_' | b'$'))
        .collect()
}

fn javascript_tokens(source: &[u8]) -> Vec<&[u8]> {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment,
        BlockComment,
        Quoted(u8),
    }

    let mut state = State::Code;
    let mut identifiers = Vec::new();
    let mut index = 0;
    while index < source.len() {
        match state {
            State::Code => {
                if source[index..].starts_with(b"//") {
                    state = State::LineComment;
                    index += 2;
                    continue;
                }
                if source[index..].starts_with(b"/*") {
                    state = State::BlockComment;
                    index += 2;
                    continue;
                }
                if matches!(source[index], b'\'' | b'"' | b'`') {
                    state = State::Quoted(source[index]);
                    index += 1;
                    continue;
                }
                if source[index].is_ascii_alphabetic() || matches!(source[index], b'_' | b'$') {
                    let start = index;
                    index += 1;
                    while source.get(index).is_some_and(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
                    }) {
                        index += 1;
                    }
                    identifiers.push(&source[start..index]);
                    continue;
                }
                if !source[index].is_ascii_whitespace() {
                    identifiers.push(&source[index..index + 1]);
                }
            }
            State::LineComment => {
                if source[index] == b'\n' {
                    state = State::Code;
                }
            }
            State::BlockComment => {
                if source[index..].starts_with(b"*/") {
                    state = State::Code;
                    index += 2;
                    continue;
                }
            }
            State::Quoted(quote) => {
                if source[index] == b'\\' {
                    index = (index + 2).min(source.len());
                    continue;
                }
                if source[index] == quote {
                    state = State::Code;
                }
            }
        }
        index += 1;
    }
    identifiers
}

pub(crate) fn component_title(component: &Path, stem: &str) -> String {
    let name = if stem == "index" {
        component_name(component)
    } else {
        stem.strip_suffix(".component").unwrap_or(stem)
    };
    format!("Components/{name}")
}

pub(crate) fn component_name(component: &Path) -> &str {
    let stem = component
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    if stem == "index" {
        component
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or(stem)
    } else {
        stem
    }
}

fn angular_export_name(stem: &str) -> String {
    let (name, suffix) = match stem.strip_suffix(".component") {
        Some(name) => (name, "Component"),
        None => (stem, ""),
    };
    let pascal_case = pascal_case_identifier(name);
    format!("{pascal_case}{suffix}")
}

fn pascal_case_identifier(name: &str) -> String {
    name.split(|character: char| !character.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().chain(characters).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn escape_javascript_string(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() || matches!(character, '\u{2028}' | '\u{2029}') => {
                escaped.push_str(&format!("\\u{:04x}", u32::from(character)));
            }
            character => escaped.push(character),
        }
    }
    escaped
}
