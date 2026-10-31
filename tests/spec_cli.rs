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
fn init_requires_framework_before_creating_files() {
    let p = TestProject::new();
    p.add_with_contents("Button.tsx", "export default function Button() {}\n");
    let out = run(&p, &["spec", "init", "--spec-dir", "specs"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("--framework"));
    assert!(!p.root.join("specs").exists());
}

#[test]
fn preact_init_generates_story_and_detects_title_collisions() {
    let p = TestProject::new();
    p.add_with_contents("src/Button.jsx", "export function Button() {}\n");
    let args = [
        "spec",
        "init",
        "src",
        "--framework",
        "preact",
        "--spec-dir",
        "specs",
        "--with-stories",
    ];
    let out = run(&p, &args);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        fs::read_to_string(p.root.join("src/Button.stories.jsx"))
            .unwrap()
            .contains("import { Button as Component }")
    );
    assert!(
        fs::read_to_string(p.root.join("specs/Button.yaml"))
            .unwrap()
            .contains("id: default")
    );

    let other = TestProject::new();
    other.add_with_contents("src/a/Button.jsx", "export function Button() {}\n");
    other.add_with_contents("src/b/Button.jsx", "export function Button() {}\n");
    let collision = run(&other, &args);
    assert_eq!(collision.status.code(), Some(2));
    assert!(stderr(&collision).contains("ambiguous"));
    assert!(!other.root.join("src/a/Button.stories.jsx").exists());
}

#[test]
fn solid_init_generates_a_story_and_rejects_title_collision() {
    let args = [
        "spec",
        "init",
        "src",
        "--framework",
        "solid",
        "--spec-dir",
        "specs",
        "--with-stories",
    ];
    let p = TestProject::new();
    p.add_with_contents("src/Button.tsx", "export function Button() {}\n");
    let out = run(&p, &args);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        fs::read_to_string(p.root.join("src/Button.stories.tsx"))
            .unwrap()
            .contains("import { Button as Component }")
    );
    assert!(
        fs::read_to_string(p.root.join("specs/Button.yaml"))
            .unwrap()
            .contains("id: default")
    );
    let other = TestProject::new();
    other.add_with_contents("src/a/Button.tsx", "export function Button() {}\n");
    other.add_with_contents("src/b/Button.tsx", "export function Button() {}\n");
    let out = run(&other, &args);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!other.root.join("src/a/Button.stories.tsx").exists());
}

#[test]
fn qwik_init_generates_story_and_rejects_title_collision() {
    let args = [
        "spec",
        "init",
        "src",
        "--framework",
        "qwik",
        "--spec-dir",
        "specs",
        "--with-stories",
    ];
    let p = TestProject::new();
    p.add_with_contents(
        "src/Button.tsx",
        "export const Button = component$(() => null);\n",
    );
    let out = run(&p, &args);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        fs::read_to_string(p.root.join("src/Button.stories.tsx"))
            .unwrap()
            .contains("import { Button as Component }")
    );
    assert!(
        fs::read_to_string(p.root.join("specs/Button.yaml"))
            .unwrap()
            .contains("id: default")
    );
    let other = TestProject::new();
    other.add_with_contents(
        "src/a/Button.tsx",
        "export const Button = component$(() => null);\n",
    );
    other.add_with_contents(
        "src/b/Button.tsx",
        "export const Button = component$(() => null);\n",
    );
    let out = run(&other, &args);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!other.root.join("src/a/Button.stories.tsx").exists());
}

#[test]
fn react_native_web_spec_init_creates_platform_story() {
    let p = TestProject::new();
    p.add_with_contents(
        "src/Button.web.tsx",
        "export default function Button() {}\n",
    );
    let out = run(
        &p,
        &[
            "spec",
            "init",
            "src",
            "--framework",
            "react",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        fs::read_to_string(p.root.join("src/Button.web.stories.tsx"))
            .unwrap()
            .contains("from './Button.web'")
    );
    assert!(
        fs::read_to_string(p.root.join("specs/Button.web.yaml"))
            .unwrap()
            .contains("Components/Button.web")
    );
}

#[test]
fn react_native_duplicate_platform_titles_fail_before_writing() {
    let p = TestProject::new();
    p.add_with_contents(
        "src/a/Button.ios.tsx",
        "export default function Button() {}\n",
    );
    p.add_with_contents(
        "src/b/Button.ios.tsx",
        "export default function Button() {}\n",
    );
    let out = run(
        &p,
        &[
            "spec",
            "init",
            "src",
            "--framework",
            "react",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!p.root.join("src/a/Button.ios.stories.tsx").exists());
    assert!(!p.root.join("src/b/Button.ios.stories.tsx").exists());
}

#[test]
fn next_app_router_duplicate_page_titles_are_rejected_without_writes() {
    let p = TestProject::new();
    p.add_with_contents("app/page.tsx", "export default function Page() {}\n");
    p.add_with_contents(
        "app/dashboard/page.tsx",
        "export default function Dashboard() {}\n",
    );
    let out = run(
        &p,
        &[
            "spec",
            "init",
            "app",
            "--framework",
            "react",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!p.root.join("app/page.stories.tsx").exists());
    assert!(!p.root.join("app/dashboard/page.stories.tsx").exists());
    assert!(!p.root.join("specs/page.yaml").exists());
}

#[test]
fn tanstack_duplicate_route_titles_are_rejected_without_writes() {
    let p = TestProject::new();
    p.add_with_contents("src/routes/about.tsx", "export const Route = {};\n");
    p.add_with_contents("src/routes/admin/about.tsx", "export const Route = {};\n");
    let out = run(
        &p,
        &[
            "spec",
            "init",
            "src",
            "--framework",
            "react",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!p.root.join("src/routes/about.stories.tsx").exists());
    assert!(!p.root.join("src/routes/admin/about.stories.tsx").exists());
}

#[test]
fn nuxt_duplicate_page_titles_are_rejected_without_writes() {
    let p = TestProject::new();
    p.add("pages/a/about.vue");
    p.add("pages/b/about.vue");
    let out = run(
        &p,
        &[
            "spec",
            "init",
            "pages",
            "--framework",
            "vue",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!p.root.join("pages/a/about.stories.ts").exists());
    assert!(!p.root.join("pages/b/about.stories.ts").exists());
}

#[test]
fn sveltekit_page_spec_ids_are_rejected_without_writes() {
    let p = TestProject::new();
    p.add("src/routes/+page.svelte");
    p.add("src/routes/admin/+page.svelte");
    let out = run(
        &p,
        &[
            "spec",
            "init",
            "src/routes",
            "--framework",
            "svelte",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("cannot be represented as a v1 id"));
    assert!(!p.root.join("src/routes/+page.stories.ts").exists());
    assert!(!p.root.join("src/routes/admin/+page.stories.ts").exists());
}

#[test]
fn sveltekit_duplicate_component_titles_are_rejected_without_writes() {
    let p = TestProject::new();
    p.add("src/lib/a/Card.svelte");
    p.add("src/lib/b/Card.svelte");
    let out = run(
        &p,
        &[
            "spec",
            "init",
            "src/lib",
            "--framework",
            "svelte",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!p.root.join("src/lib/a/Card.stories.ts").exists());
    assert!(!p.root.join("src/lib/b/Card.stories.ts").exists());
}

#[test]
fn analog_duplicate_component_titles_are_rejected_without_writes() {
    let p = TestProject::new();
    p.add("src/app/a/card.component.ts");
    p.add("src/app/b/card.component.ts");
    let out = run(
        &p,
        &[
            "spec",
            "init",
            "src/app",
            "--framework",
            "angular",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!p.root.join("src/app/a/card.component.stories.ts").exists());
    assert!(!p.root.join("src/app/b/card.component.stories.ts").exists());
}

#[test]
fn web_components_spec_init_generates_and_rejects_title_collision() {
    let p = TestProject::new();
    p.add_with_contents("src/card.ts", "@customElement('my-card') class Card {}\n");
    let args = [
        "spec",
        "init",
        "src",
        "--framework",
        "web-components",
        "--spec-dir",
        "specs",
        "--with-stories",
    ];
    let out = run(&p, &args);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        fs::read_to_string(p.root.join("src/card.stories.ts"))
            .unwrap()
            .contains("document.createElement('my-card')")
    );
    assert!(
        fs::read_to_string(p.root.join("specs/card.yaml"))
            .unwrap()
            .contains("id: default")
    );
    let other = TestProject::new();
    other.add_with_contents("src/a/card.js", "customElements.define('a-card', Card);\n");
    other.add_with_contents("src/b/card.js", "customElements.define('b-card', Card);\n");
    let out = run(&other, &args);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!other.root.join("src/a/card.stories.js").exists());
}

#[test]
fn html_spec_init_generates_story_and_rejects_colliding_titles() {
    let args = [
        "spec",
        "init",
        "src",
        "--framework",
        "html",
        "--spec-dir",
        "specs",
        "--with-stories",
    ];
    let p = TestProject::new();
    p.add_with_contents("src/card.component.html", "<p>Card</p>");
    let out = run(&p, &args);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        fs::read_to_string(p.root.join("src/card.component.stories.js"))
            .unwrap()
            .contains("./card.component.html?raw")
    );
    let specs = fs::read_dir(p.root.join("specs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(fs::read_to_string(specs).unwrap().contains("id: default"));
    let other = TestProject::new();
    other.add("src/a/card.component.html");
    other.add("src/b/card.component.html");
    let out = run(&other, &args);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!other.root.join("src/a/card.component.stories.js").exists());
}

#[test]
fn server_index_checks_declared_states_but_does_not_discover_components() {
    let p = TestProject::new();
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/server-express");
    let index = fixture.join("index.sample.json");
    let only_button = fixture.join("specs-button");
    let both = fixture.join("specs-both");
    let pass = run(
        &p,
        &[
            "spec",
            "check",
            "--spec-dir",
            only_button.to_str().unwrap(),
            "--index",
            index.to_str().unwrap(),
        ],
    );
    assert_eq!(pass.status.code(), Some(0), "{}", stderr(&pass));
    assert!(stdout(&pass).contains("Components checked: 1"));
    let missing = run(
        &p,
        &[
            "spec",
            "check",
            "--spec-dir",
            both.to_str().unwrap(),
            "--index",
            index.to_str().unwrap(),
        ],
    );
    assert_eq!(missing.status.code(), Some(1));
    assert!(stdout(&missing).contains("Card/default"));
}

#[test]
fn ember_spec_init_generates_and_rejects_title_collisions() {
    let args = [
        "spec",
        "init",
        "app/components",
        "--framework",
        "ember",
        "--spec-dir",
        "specs",
        "--with-stories",
    ];
    let p = TestProject::new();
    p.add("app/components/card.gjs");
    let out = run(&p, &args);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        fs::read_to_string(p.root.join("app/components/card.stories.js"))
            .unwrap()
            .contains("<Card />")
    );
    assert!(
        fs::read_to_string(p.root.join("specs/card.yaml"))
            .unwrap()
            .contains("id: default")
    );
    let other = TestProject::new();
    other.add("app/components/a/card.gjs");
    other.add("app/components/b/card.gjs");
    let out = run(&other, &args);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("ambiguous"));
    assert!(!other.root.join("app/components/a/card.stories.js").exists());
}

#[test]
fn svelte_init_generates_matching_specs_and_preserves_native_stories() {
    let p = TestProject::new();
    p.add_with_contents("Button.svelte", "<button>Button</button>\n");
    p.add_with_contents("Card.svelte", "<div>Card</div>\n");
    p.add_with_contents("Card.stories.svelte", "<!-- existing story -->\n");
    let out = run(
        &p,
        &[
            "spec",
            "init",
            ".",
            "--framework",
            "svelte",
            "--spec-dir",
            "specs",
            "--with-stories",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let yaml = fs::read_to_string(p.root.join("specs/Button.yaml")).unwrap();
    assert!(yaml.contains("title: Components/Button"));
    assert!(yaml.contains("id: default"));
    let story = fs::read_to_string(p.root.join("Button.stories.ts")).unwrap();
    assert!(story.contains("import Component from './Button.svelte';"));
    assert!(story.contains("title: 'Components/Button'"));
    assert!(story.contains("export const Default = {}"));
    assert!(!p.root.join("Card.stories.ts").exists());
    assert!(
        fs::read_to_string(p.root.join("specs/Card.yaml"))
            .unwrap()
            .contains("stories: []")
    );
    assert_eq!(
        fs::read_to_string(p.root.join("Card.stories.svelte")).unwrap(),
        "<!-- existing story -->\n"
    );
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
    let out = run(
        &p,
        &[
            "spec",
            "init",
            root,
            "--framework",
            "react",
            "--spec-dir",
            spec_dir,
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let draft = fs::read_to_string(p.root.join("specs/Button.yaml")).unwrap();
    assert!(draft.contains("stories: []"));
    let again = run(
        &p,
        &[
            "spec",
            "init",
            root,
            "--framework",
            "react",
            "--spec-dir",
            spec_dir,
        ],
    );
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
        &[
            "spec",
            "init",
            path,
            "--framework",
            "react",
            "--spec-dir",
            dir,
            "--with-stories",
        ],
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
fn merge_updates_generated_specs_for_all_frameworks() {
    let cases = [
        (
            "react",
            "Button.tsx",
            "export default function Button() {}",
            "Button.stories.tsx",
        ),
        (
            "vue",
            "Button.vue",
            "<template><button /></template>",
            "Button.stories.ts",
        ),
        ("svelte", "Button.svelte", "<button />", "Button.stories.ts"),
        (
            "angular",
            "Button.component.ts",
            "export class ButtonComponent {}",
            "Button.component.stories.ts",
        ),
    ];
    for (framework, component, source, story_path) in cases {
        for from_init in [false, true] {
            let p = TestProject::new();
            p.add_with_contents(&format!("src/{component}"), source);
            let write_index = |names: &[&str]| {
                let entries = names
                    .iter()
                    .map(|name| {
                        let id = format!("components-button--{}", name.to_lowercase());
                        (
                            id.clone(),
                            serde_json::json!({
                                "id": id, "type": "story", "title": "Components/Button",
                                "name": name, "importPath": format!("./src/{story_path}")
                            }),
                        )
                    })
                    .collect::<serde_json::Map<_, _>>();
                p.add_with_contents(
                    "index.json",
                    &serde_json::json!({"v": 5, "entries": entries}).to_string(),
                );
            };
            write_index(&["Default"]);
            let generated = if from_init {
                run(
                    &p,
                    &[
                        "spec",
                        "init",
                        "src",
                        "--framework",
                        framework,
                        "--spec-dir",
                        "specs",
                        "--with-stories",
                    ],
                )
            } else {
                run(
                    &p,
                    &[
                        "spec",
                        "import",
                        "--index",
                        "index.json",
                        "--spec-dir",
                        "specs",
                    ],
                )
            };
            assert_eq!(
                generated.status.code(),
                Some(0),
                "{framework}, init={from_init}: {}",
                stderr(&generated)
            );
            let spec_name = if from_init {
                format!("specs/{}.yaml", component.rsplit_once('.').unwrap().0)
            } else {
                "specs/components-button.yaml".into()
            };
            let path = p.root.join(spec_name);
            let original = fs::read_to_string(&path).unwrap();
            let merge_args = [
                "spec",
                "import",
                "--index",
                "index.json",
                "--spec-dir",
                "specs",
                "--merge",
            ];
            let unchanged = run(&p, &merge_args);
            assert_eq!(unchanged.status.code(), Some(0), "{}", stderr(&unchanged));
            assert!(stdout(&unchanged).contains("Skipped 1"));
            assert_eq!(fs::read_to_string(&path).unwrap(), original);

            write_index(&["Default", "Failure", "Loading"]);
            let merged = run(&p, &merge_args);
            assert_eq!(
                merged.status.code(),
                Some(0),
                "{framework}, init={from_init}: {}",
                stderr(&merged)
            );
            let updated = fs::read_to_string(&path).unwrap();
            assert!(updated.starts_with(&original));
            let yaml: serde_yaml::Value = serde_yaml::from_str(&updated).unwrap();
            let stories = yaml["stories"].as_sequence().unwrap();
            assert_eq!(stories.len(), 3);
            for (story, id) in stories.iter().zip(["default", "failure", "loading"]) {
                assert_eq!(story["id"].as_str(), Some(id));
                assert_eq!(story["required"].as_bool(), Some(true));
            }
            let checked = run(
                &p,
                &[
                    "spec",
                    "check",
                    "--index",
                    "index.json",
                    "--spec-dir",
                    "specs",
                ],
            );
            assert_eq!(checked.status.code(), Some(0), "{}", stderr(&checked));
            assert!(stdout(&checked).contains("No issues found."));

            let again = run(&p, &merge_args);
            assert_eq!(again.status.code(), Some(0), "{}", stderr(&again));
            assert!(stdout(&again).contains("Skipped 1"));
            assert_eq!(fs::read_to_string(&path).unwrap(), updated);
        }
    }
}

#[test]
fn merge_preserves_comments_and_optional_values() {
    for indent in ["", "  "] {
        let p = TestProject::new();
        let original = "# hand written\nversion: 1\ncomponent:\n  id: button\n  name: Button\n  title: Components/Button\nstories:\n  - id: default # keep\n    required: false\n";
        let original = original
            .replace("  - id:", &format!("{indent}- id:"))
            .replace("    required:", &format!("{indent}  required:"));
        p.add_with_contents("specs/button.yaml", &original);
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
        assert!(updated.starts_with(&original));
        assert!(updated.contains(&format!(
            "{indent}- id: loading\n{indent}  required: true\n"
        )));
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
            "--framework",
            "react",
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
            "--framework",
            "react",
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
