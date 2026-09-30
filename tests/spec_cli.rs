mod support;

use std::{
    fs,
    process::{Command, Output},
};
use support::TestProject;

fn run(project: &TestProject, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_storymesh"))
        .current_dir(&project.root)
        .args(args)
        .output()
        .expect("binary runs")
}
fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}
fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
fn spec(stories: &str, title: &str) -> String {
    format!("version: 1\ncomponent:\n  id: button\n  name: Button\n{title}stories:\n{stories}")
}
fn index(entries: &str) -> String {
    format!("{{\"entries\":{{{entries}}}}}")
}

#[test]
fn check_reports_missing_and_warning_in_stable_order() {
    let p = TestProject::new();
    p.add_with_contents(
        "specs/button.yaml",
        &spec(
            "  - id: loading\n  - id: default\n  - id: optional\n    required: false\n",
            "  title: Components/Button\n",
        ),
    );
    p.add_with_contents("index.json", &index("\"z\":{\"id\":\"z\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Debug\"},\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"},\"doc\":{\"type\":\"docs\"},\"other\":{\"id\":\"other\",\"type\":\"story\",\"title\":\"Elsewhere/Card\",\"name\":\"Default\"}"));
    let out = run(
        &p,
        &[
            "spec",
            "check",
            "--spec-dir",
            "specs",
            "--index",
            "index.json",
        ],
    );
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stdout(&out).contains("Button/loading"));
    assert!(stdout(&out).contains("Button/debug"));
    assert!(!stdout(&out).contains("Card/default"));
    assert!(stdout(&out).contains("Stories checked:    2"));
    assert!(stdout(&out).ends_with("FAIL\n"));
}

#[test]
fn check_passes_when_required_exists_and_optional_is_absent() {
    let p = TestProject::new();
    p.add_with_contents(
        "specs/button.yaml",
        &spec(
            "  - id: default\n  - id: optional\n    required: false\n",
            "  title: Components/Button\n",
        ),
    );
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"}"));
    let out = run(
        &p,
        &[
            "spec",
            "check",
            "--spec-dir",
            "specs",
            "--index",
            "index.json",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(stdout(&out).contains("No issues found."));
    assert!(stdout(&out).ends_with("PASS\n"));
}

#[test]
fn check_rejects_ambiguous_and_invalid_input() {
    let p = TestProject::new();
    p.add_with_contents("specs/button.yaml", &spec("  - id: default\n", ""));
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"},\"b\":{\"id\":\"b\",\"type\":\"story\",\"title\":\"Forms/Button\",\"name\":\"Default\"}"));
    let args = [
        "spec",
        "check",
        "--spec-dir",
        "specs",
        "--index",
        "index.json",
    ];
    let ambiguous = run(&p, &args);
    assert_eq!(ambiguous.status.code(), Some(2));
    assert!(stderr(&ambiguous).contains("component.title"));
    assert!(ambiguous.stdout.is_empty());
    p.add_with_contents(
        "specs/button.yaml",
        &spec("  - id: default\n", "  title: Components/Button\n"),
    );
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"},\"b\":{\"id\":\"b\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"}"));
    let collision = run(&p, &args);
    assert_eq!(collision.status.code(), Some(2));
    assert!(stderr(&collision).contains("ambiguous story name"));
    p.add_with_contents(
        "specs/button.yaml",
        &format!(
            "{}scenarios: []\n",
            spec("  - id: default\n", "  title: Components/Button\n")
        ),
    );
    let unknown = run(&p, &args);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(stderr(&unknown).contains("scenarios"));
}

#[test]
fn init_creates_draft_and_with_stories_creates_matching_pair() {
    let p = TestProject::new();
    p.add_with_contents(
        "components/Button.tsx",
        "export default function Button() {}\n",
    );
    let root = p.root.join("components");
    let root = root.to_str().unwrap();
    let spec_dir = p.root.join("specs");
    let spec_dir = spec_dir.to_str().unwrap();
    let out = run(&p, &["spec", "init", root, "--spec-dir", spec_dir]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let draft = fs::read_to_string(p.root.join("specs/Button.yaml")).unwrap();
    assert!(draft.contains("stories: []"));
    let again = run(&p, &["spec", "init", root, "--spec-dir", spec_dir]);
    assert_eq!(again.status.code(), Some(0), "{}", stderr(&again));
    assert!(stdout(&again).contains("Skipped 1"));
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"}"));
    let check = run(
        &p,
        &[
            "spec",
            "check",
            "--spec-dir",
            spec_dir,
            "--index",
            "index.json",
        ],
    );
    assert_eq!(check.status.code(), Some(2));
    assert!(stderr(&check).contains("draft"));

    let q = TestProject::new();
    q.add_with_contents("Button.tsx", "export default function Button() {}\n");
    let path = q.root.to_str().unwrap();
    let dir = q.root.join("specs");
    let dir = dir.to_str().unwrap();
    let made = run(
        &q,
        &["spec", "init", path, "--spec-dir", dir, "--with-stories"],
    );
    assert_eq!(made.status.code(), Some(0), "{}", stderr(&made));
    assert!(q.root.join("Button.stories.tsx").exists());
    let yaml = fs::read_to_string(q.root.join("specs/Button.yaml")).unwrap();
    assert!(yaml.contains("title: Components/Button"));
    assert!(yaml.contains("id: default"));
}

#[test]
fn import_creates_per_title_specs_without_overwriting() {
    let p = TestProject::new();
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"},\"b\":{\"id\":\"b\",\"type\":\"story\",\"title\":\"Forms/Input\",\"name\":\"Loading State\"},\"doc\":{\"type\":\"docs\"}"));
    let first = run(
        &p,
        &[
            "spec",
            "import",
            "--index",
            "index.json",
            "--spec-dir",
            "specs",
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{}", stderr(&first));
    let path = p.root.join("specs/components-button.yaml");
    assert!(fs::read_to_string(&path).unwrap().contains("id: default"));
    let second = run(
        &p,
        &[
            "spec",
            "import",
            "--index",
            "index.json",
            "--spec-dir",
            "specs",
        ],
    );
    assert_eq!(second.status.code(), Some(0), "{}", stderr(&second));
    assert!(stdout(&second).contains("Skipped 2"));
    assert!(p.root.join("specs/forms-input.yaml").exists());
}

#[test]
fn warning_only_passes_and_missing_inputs_fail_without_pass() {
    let p = TestProject::new();
    p.add("unrelated.txt");
    p.add_with_contents(
        "specs/button.yaml",
        &spec("  - id: default\n", "  title: Components/Button\n"),
    );
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"},\"b\":{\"id\":\"b\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Debug\"}"));
    let args = [
        "spec",
        "check",
        "--spec-dir",
        "specs",
        "--index",
        "index.json",
    ];
    let warning = run(&p, &args);
    assert_eq!(warning.status.code(), Some(0));
    assert!(stdout(&warning).contains("Button/debug"));
    assert!(stdout(&warning).contains("WARNING"));
    assert!(stdout(&warning).ends_with("PASS\n"));
    fs::remove_file(p.root.join("index.json")).unwrap();
    let missing_index = run(&p, &args);
    assert_eq!(missing_index.status.code(), Some(2));
    assert!(missing_index.stdout.is_empty());
    assert!(stderr(&missing_index).contains("index.json"));
    p.add_with_contents("index.json", "{");
    let broken_json = run(&p, &args);
    assert_eq!(broken_json.status.code(), Some(2));
    assert!(stderr(&broken_json).contains("index.json"));
}

#[test]
fn merge_preserves_comments_and_optional_values() {
    let p = TestProject::new();
    let original = "# hand written\nversion: 1\ncomponent:\n  id: button\n  name: Button\n  title: Components/Button\nstories:\n  - id: default # keep\n    required: false\n";
    p.add_with_contents("specs/button.yaml", original);
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"},\"b\":{\"id\":\"b\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Loading\"}"));
    let out = run(
        &p,
        &[
            "spec",
            "import",
            "--index",
            "index.json",
            "--spec-dir",
            "specs",
            "--merge",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let updated = fs::read_to_string(p.root.join("specs/button.yaml")).unwrap();
    assert!(updated.starts_with(original));
    assert!(updated.contains("  - id: loading\n    required: true\n"));
    let again = run(
        &p,
        &[
            "spec",
            "import",
            "--index",
            "index.json",
            "--spec-dir",
            "specs",
            "--merge",
        ],
    );
    assert_eq!(again.status.code(), Some(0), "{}", stderr(&again));
    assert!(stdout(&again).contains("Skipped 1"));
    assert_eq!(
        fs::read_to_string(p.root.join("specs/button.yaml")).unwrap(),
        updated
    );
}

#[test]
fn import_collision_is_preflighted_before_any_write() {
    let p = TestProject::new();
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"},\"b\":{\"id\":\"b\",\"type\":\"story\",\"title\":\"Forms/Input\",\"name\":\"Loading\"}"));
    let existing = "version: 1\ncomponent:\n  id: unrelated\n  name: Other\n  title: Other/Thing\nstories:\n  - id: default\n";
    p.add_with_contents("specs/forms-input.yaml", existing);
    let out = run(
        &p,
        &[
            "spec",
            "import",
            "--index",
            "index.json",
            "--spec-dir",
            "specs",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(!p.root.join("specs/components-button.yaml").exists());
    assert_eq!(
        fs::read_to_string(p.root.join("specs/forms-input.yaml")).unwrap(),
        existing
    );
}

#[test]
fn angular_init_uses_distinct_component_suffix_path_and_matching_title() {
    let p = TestProject::new();
    p.add_with_contents(
        "src/user-card.component.ts",
        "export class UserCardComponent {}\n",
    );
    let root = p.root.join("src");
    let out = run(
        &p,
        &[
            "spec",
            "init",
            root.to_str().unwrap(),
            "--framework",
            "angular",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let yaml = fs::read_to_string(p.root.join("specs/user-card.component.yaml")).unwrap();
    assert!(yaml.contains("id: user-card-component"));
    assert!(yaml.contains("name: user-card"));
    assert!(yaml.contains("title: Components/user-card"));
    assert!(p.root.join("src/user-card.component.stories.ts").exists());
    assert!(stdout(&out).contains("storymesh spec check"));
}

#[test]
fn init_with_existing_story_keeps_it_and_creates_a_draft() {
    let p = TestProject::new();
    p.add_with_contents("Button.tsx", "export default function Button() {}\n");
    p.add_with_contents("Button.stories.tsx", "// existing hand-written story\n");
    let out = run(
        &p,
        &[
            "spec",
            "init",
            p.root.to_str().unwrap(),
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(
        fs::read_to_string(p.root.join("Button.stories.tsx")).unwrap(),
        "// existing hand-written story\n"
    );
    assert!(stdout(&out).contains("storymesh spec import"));
    assert!(
        fs::read_to_string(p.root.join("specs/Button.yaml"))
            .unwrap()
            .contains("stories: []")
    );
}

#[test]
fn merge_rejects_flow_list_without_touching_original() {
    let p = TestProject::new();
    let original = "version: 1\ncomponent:\n  id: button\n  name: Button\n  title: Components/Button\nstories: [{id: default, required: false}]\n";
    p.add_with_contents("specs/button.yaml", original);
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Loading\"}"));
    let out = run(
        &p,
        &[
            "spec",
            "import",
            "--index",
            "index.json",
            "--spec-dir",
            "specs",
            "--merge",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("block-form"));
    assert_eq!(
        fs::read_to_string(p.root.join("specs/button.yaml")).unwrap(),
        original
    );
}

#[test]
fn empty_and_duplicate_spec_inputs_are_errors() {
    let p = TestProject::new();
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Default\"}"));
    let args = [
        "spec",
        "check",
        "--spec-dir",
        "specs",
        "--index",
        "index.json",
    ];
    let empty = run(&p, &args);
    assert_eq!(empty.status.code(), Some(2));
    assert!(stderr(&empty).contains("specs"));
    p.add_with_contents(
        "specs/one.yaml",
        &spec("  - id: default\n", "  title: Components/Button\n"),
    );
    p.add_with_contents(
        "specs/two.yaml",
        &spec("  - id: loading\n", "  title: Forms/Button\n"),
    );
    let duplicate = run(&p, &args);
    assert_eq!(duplicate.status.code(), Some(2));
    assert!(stderr(&duplicate).contains("duplicates"));
}

#[test]
fn init_title_collision_writes_nothing() {
    let p = TestProject::new();
    p.add_with_contents(
        "src/forms/Button.tsx",
        "export default function Button() {}\n",
    );
    p.add_with_contents(
        "src/controls/Button.tsx",
        "export default function Button() {}\n",
    );
    let root = p.root.join("src");
    let out = run(
        &p,
        &[
            "spec",
            "init",
            root.to_str().unwrap(),
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!p.root.join("src/forms/Button.stories.tsx").exists());
    assert!(!p.root.join("specs/controls/Button.yaml").exists());
}

#[test]
fn check_output_is_independent_of_file_and_entry_order() {
    let p = TestProject::new();
    let button = spec(
        "  - id: loading\n  - id: default\n",
        "  title: Components/Button\n",
    );
    let card = "version: 1\ncomponent:\n  id: card\n  name: Card\n  title: Components/Card\nstories:\n  - id: error\n";
    p.add_with_contents("specs/z-button.yaml", &button);
    p.add_with_contents("specs/a-card.yaml", card);
    let first_index = index(
        "\"z\":{\"id\":\"z\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Debug\"},\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Card\",\"name\":\"Default\"}",
    );
    p.add_with_contents("index.json", &first_index);
    let args = [
        "spec",
        "check",
        "--spec-dir",
        "specs",
        "--index",
        "index.json",
    ];
    let first = run(&p, &args);
    assert_eq!(first.status.code(), Some(1));
    fs::rename(
        p.root.join("specs/z-button.yaml"),
        p.root.join("specs/0-button.yaml"),
    )
    .unwrap();
    fs::rename(
        p.root.join("specs/a-card.yaml"),
        p.root.join("specs/z-card.yaml"),
    )
    .unwrap();
    p.add_with_contents("index.json", &index("\"a\":{\"id\":\"a\",\"type\":\"story\",\"title\":\"Components/Card\",\"name\":\"Default\"},\"z\":{\"id\":\"z\",\"type\":\"story\",\"title\":\"Components/Button\",\"name\":\"Debug\"}"));
    let second = run(&p, &args);
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn import_invalid_title_identifies_storybook_entry() {
    let p = TestProject::new();
    p.add_with_contents("index.json", &index("\"entry\":{\"id\":\"button--default\",\"type\":\"story\",\"title\":\"Components/Button!\",\"name\":\"Default\"}"));
    let out = run(
        &p,
        &[
            "spec",
            "import",
            "--index",
            "index.json",
            "--spec-dir",
            "specs",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    let error = stderr(&out);
    assert!(error.contains("button--default"));
    assert!(error.contains("Components/Button!"));
    assert!(error.contains("Default"));
    assert!(!p.root.join("specs").exists());
}
