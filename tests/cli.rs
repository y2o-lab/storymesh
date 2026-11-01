mod support;

use std::process::Command;

use support::TestProject;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_storymesh"))
        .args(args)
        .output()
        .expect("the storymesh binary should run")
}

#[test]
fn check_lists_missing_components_and_returns_one() {
    let project = TestProject::new();
    project.add("Button.tsx");
    project.add("Card.tsx");
    project.add("Button.stories.tsx");

    let output = run(&["check", project.root.to_str().expect("UTF-8 test path")]);

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Missing stories for 1 React component(s):"));
    assert!(stdout.contains("Card.tsx"));
    assert!(!stdout.contains("Button.tsx\n"));
}

#[test]
fn coverage_and_report_have_success_exit_codes() {
    let project = TestProject::new();
    project.add("Card.vue");

    let root = project.root.to_str().expect("UTF-8 test path");
    let coverage = run(&["coverage", root, "--framework", "vue"]);
    assert_eq!(coverage.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&coverage.stdout).contains("Vue Storybook coverage: 0.0%"));

    let report = run(&["report", root, "--framework", "vue"]);
    assert_eq!(report.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&report.stdout).contains("Missing: 1"));
}

#[test]
fn invalid_roots_return_an_actionable_error() {
    let output = run(&["check", "directory-that-does-not-exist"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("directory-that-does-not-exist"));
}

#[test]
fn check_generate_writes_story_skeletons_and_returns_zero() {
    let project = TestProject::new();
    project.add_with_contents("Card.tsx", "export default function Card() {}\n");

    let output = run(&[
        "check",
        project.root.to_str().expect("UTF-8 test path"),
        "--generate",
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(project.root.join("Card.stories.tsx").is_file());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generated 1 story skeleton(s):"));
    assert!(stdout.contains("Card.stories.tsx"));
}

#[test]
fn check_generate_returns_two_when_a_story_target_exists() {
    let project = TestProject::new();
    project.add_with_contents("Card.tsx", "export default function Card() {}\n");
    std::fs::create_dir(project.root.join("Card.stories.tsx"))
        .expect("the conflicting directory should be created");

    let output = run(&[
        "check",
        project.root.to_str().expect("UTF-8 test path"),
        "--generate",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
}

#[test]
fn check_excludes_patterns_and_additional_ignore_files() {
    let project = TestProject::new();
    project.add("Button.tsx");
    project.add("Generated.tsx");
    project.add("Legacy.tsx");
    project.add_with_contents("custom.ignore", "Legacy.tsx\n");

    let output = run(&[
        "check",
        project.root.to_str().expect("UTF-8 test path"),
        "--ignore",
        "Generated.tsx",
        "--ignore-file",
        "custom.ignore",
    ]);

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Button.tsx"));
    assert!(!stdout.contains("Generated.tsx"));
    assert!(!stdout.contains("Legacy.tsx"));
}

#[test]
fn svelte_commands_report_coverage_and_generate_only_missing_stories() {
    let project = TestProject::new();
    project.add_with_contents("Button.svelte", "<button>Button</button>\n");
    let native_story = "<!-- existing native story -->\n";
    project.add_with_contents("Button.stories.svelte", native_story);
    project.add_with_contents("Card.svelte", "<div>Card</div>\n");
    let root = project.root.to_str().expect("UTF-8 test path");

    let check = run(&["check", root, "--framework", "svelte"]);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("1 Svelte component(s):\nCard.svelte"));
    for command in ["coverage", "report"] {
        let output = run(&[command, root, "--framework", "svelte"]);
        assert_eq!(output.status.code(), Some(0));
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("Svelte Storybook coverage: 50.0% (1/2 components)")
        );
    }

    let generated = run(&["check", root, "--framework", "svelte", "--generate"]);
    assert_eq!(generated.status.code(), Some(0));
    let contents = std::fs::read_to_string(project.root.join("Card.stories.ts"))
        .expect("generated story reads");
    assert!(contents.contains("import Component from './Card.svelte';"));
    assert!(!project.root.join("Button.stories.ts").exists());
    assert_eq!(
        std::fs::read_to_string(project.root.join("Button.stories.svelte")).unwrap(),
        native_story
    );
    let recheck = run(&["check", root, "--framework", "svelte"]);
    assert_eq!(recheck.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&recheck.stdout).contains("All 2 Svelte components have stories.")
    );
}

#[test]
fn preact_commands_cover_and_generate_without_touching_existing_stories() {
    let project = TestProject::new();
    project.add_with_contents("Button.tsx", "export default function Button() {}\n");
    project.add_with_contents("Button.stories.js", "// existing story\n");
    project.add_with_contents("Card.jsx", "export function Card() {}\n");
    project.add_with_contents("Ignored.tsx", "export default function Ignored() {}\n");
    let root = project.root.to_str().expect("UTF-8 test path");
    let check = run(&[
        "check",
        root,
        "--framework",
        "preact",
        "--ignore",
        "Ignored.tsx",
    ]);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("Card.jsx"));
    for command in ["coverage", "report"] {
        let result = run(&[
            command,
            root,
            "--framework",
            "preact",
            "--ignore",
            "Ignored.tsx",
        ]);
        assert_eq!(result.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&result.stdout).contains("50.0% (1/2 components)"));
    }
    let generated = run(&[
        "check",
        root,
        "--framework",
        "preact",
        "--ignore",
        "Ignored.tsx",
        "--generate",
    ]);
    assert_eq!(generated.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(project.root.join("Card.stories.jsx"))
            .unwrap()
            .contains("import { Card as Component }")
    );
    assert_eq!(
        std::fs::read_to_string(project.root.join("Button.stories.js")).unwrap(),
        "// existing story\n"
    );
}

#[test]
fn preact_generate_refuses_an_existing_target() {
    let project = TestProject::new();
    project.add_with_contents("Card.jsx", "export function Card() {}\n");
    std::fs::create_dir(project.root.join("Card.stories.jsx")).unwrap();
    let result = run(&[
        "check",
        project.root.to_str().unwrap(),
        "--framework",
        "preact",
        "--generate",
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("already exists"));
    assert!(project.root.join("Card.stories.jsx").is_dir());
}

#[test]
fn solid_commands_report_and_generate_without_overwriting() {
    let p = TestProject::new();
    p.add_with_contents("Button.tsx", "export default function Button() {}\n");
    p.add_with_contents("Button.stories.ts", "// preserve\n");
    p.add_with_contents("Card.tsx", "export function Card() {}\n");
    let root = p.root.to_str().unwrap();
    assert_eq!(
        run(&["check", root, "--framework", "solid"]).status.code(),
        Some(1)
    );
    for command in ["coverage", "report"] {
        let out = run(&[command, root, "--framework", "solid"]);
        assert_eq!(out.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&out.stdout).contains("50.0% (1/2 components)"));
    }
    let out = run(&["check", root, "--framework", "solid", "--generate"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(p.root.join("Card.stories.tsx").is_file());
    assert_eq!(
        std::fs::read_to_string(p.root.join("Button.stories.ts")).unwrap(),
        "// preserve\n"
    );
}

#[test]
fn solid_generate_refuses_an_existing_target() {
    let p = TestProject::new();
    p.add_with_contents("Card.tsx", "export function Card() {}\n");
    std::fs::create_dir(p.root.join("Card.stories.tsx")).unwrap();
    let out = run(&[
        "check",
        p.root.to_str().unwrap(),
        "--framework",
        "solid",
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("already exists"));
    assert!(p.root.join("Card.stories.tsx").is_dir());
}

#[test]
fn qwik_commands_report_generate_and_preserve_existing_stories() {
    let p = TestProject::new();
    p.add_with_contents("Button.tsx", "export default component$(() => null);\n");
    p.add_with_contents("Button.stories.ts", "// preserve\n");
    p.add_with_contents("Card.tsx", "export const Card = component$(() => null);\n");
    let root = p.root.to_str().unwrap();
    assert_eq!(
        run(&["check", root, "--framework", "qwik"]).status.code(),
        Some(1)
    );
    for command in ["coverage", "report"] {
        let out = run(&[command, root, "--framework", "qwik"]);
        assert_eq!(out.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&out.stdout).contains("50.0% (1/2 components)"));
    }
    let out = run(&["check", root, "--framework", "qwik", "--generate"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(p.root.join("Card.stories.tsx").is_file());
    assert_eq!(
        std::fs::read_to_string(p.root.join("Button.stories.ts")).unwrap(),
        "// preserve\n"
    );
}

#[test]
fn qwik_generate_refuses_an_existing_target() {
    let p = TestProject::new();
    p.add_with_contents("Card.tsx", "export const Card = component$(() => null);\n");
    std::fs::create_dir(p.root.join("Card.stories.tsx")).unwrap();
    let out = run(&[
        "check",
        p.root.to_str().unwrap(),
        "--framework",
        "qwik",
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("already exists"));
}

#[test]
fn react_native_web_commands_handle_platform_stories_and_ignores() {
    let p = TestProject::new();
    p.add_with_contents("Button.web.tsx", "export default function Button() {}\n");
    p.add_with_contents("Button.web.stories.ts", "// existing web story\n");
    p.add_with_contents("Card.web.tsx", "export default function Card() {}\n");
    p.add_with_contents("Card.ios.tsx", "export default function Card() {}\n");
    let root = p.root.to_str().unwrap();
    let args = [root, "--framework", "react", "--ignore", "*.ios.tsx"];
    let check = run(&["check", args[0], args[1], args[2], args[3], args[4]]);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("Card.web.tsx"));
    for command in ["coverage", "report"] {
        let out = run(&[command, args[0], args[1], args[2], args[3], args[4]]);
        assert_eq!(out.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&out.stdout).contains("50.0% (1/2 components)"));
    }
    let out = run(&[
        "check",
        args[0],
        args[1],
        args[2],
        args[3],
        args[4],
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("Card.web.stories.tsx"))
            .unwrap()
            .contains("from './Card.web'")
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("Button.web.stories.ts")).unwrap(),
        "// existing web story\n"
    );
    assert!(!p.root.join("Card.ios.stories.tsx").exists());
}

#[test]
fn react_native_platform_files_generate_distinct_stories_and_preserve_existing() {
    let p = TestProject::new();
    p.add_with_contents("Button.tsx", "export default function Button() {}\n");
    p.add_with_contents("Button.ios.tsx", "export default function Button() {}\n");
    p.add_with_contents("Button.native.tsx", "export default function Button() {}\n");
    p.add_with_contents(
        "Button.android.tsx",
        "export default function Button() {}\n",
    );
    p.add_with_contents("Button.ios.stories.tsx", "// existing ios story\n");
    let root = p.root.to_str().unwrap();
    let out = run(&[
        "check",
        root,
        "--framework",
        "react",
        "--ignore",
        "*.android.tsx",
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("Button.stories.tsx"))
            .unwrap()
            .contains("from './Button'")
    );
    assert!(
        std::fs::read_to_string(p.root.join("Button.native.stories.tsx"))
            .unwrap()
            .contains("from './Button.native'")
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("Button.ios.stories.tsx")).unwrap(),
        "// existing ios story\n"
    );
    assert!(!p.root.join("Button.android.stories.tsx").exists());
}

#[test]
fn next_app_router_check_can_ignore_routes_and_preserves_existing_stories() {
    let p = TestProject::new();
    p.add_with_contents(
        "app/components/Button.tsx",
        "export default function Button() {}\n",
    );
    p.add_with_contents("app/components/Button.stories.tsx", "// existing\n");
    p.add_with_contents(
        "app/components/Card.tsx",
        "export default function Card() {}\n",
    );
    p.add_with_contents("app/page.tsx", "export default function Page() {}\n");
    let root = p.root.to_str().unwrap();
    let check = run(&[
        "check",
        root,
        "--framework",
        "react",
        "--ignore",
        "page.tsx",
    ]);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("Card.tsx"));
    assert!(!String::from_utf8_lossy(&check.stdout).contains("page.tsx"));
    let generated = run(&[
        "check",
        root,
        "--framework",
        "react",
        "--ignore",
        "page.tsx",
        "--generate",
    ]);
    assert_eq!(generated.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("app/components/Card.stories.tsx"))
            .unwrap()
            .contains("from './Card'")
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("app/components/Button.stories.tsx")).unwrap(),
        "// existing\n"
    );
    assert!(!p.root.join("app/page.stories.tsx").exists());
}

#[test]
fn tanstack_routes_can_be_excluded_and_route_exports_generate_placeholders() {
    let p = TestProject::new();
    p.add_with_contents(
        "src/components/Button.tsx",
        "export default function Button() {}\n",
    );
    p.add_with_contents("src/components/Button.stories.tsx", "// existing\n");
    p.add_with_contents("src/components/Card.tsx", "export function Card() {}\n");
    p.add_with_contents(
        "src/routes/about.tsx",
        "export const Route = createFileRoute('/about')({});\n",
    );
    let root = p.root.to_str().unwrap();
    let check = run(&[
        "check",
        root,
        "--framework",
        "react",
        "--ignore",
        "src/routes/**",
    ]);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("Card.tsx"));
    assert!(!String::from_utf8_lossy(&check.stdout).contains("about.tsx"));
    let generated = run(&["check", root, "--framework", "react", "--generate"]);
    assert_eq!(generated.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("src/routes/about.stories.tsx"))
            .unwrap()
            .contains("render: () => null")
    );
    assert!(
        std::fs::read_to_string(p.root.join("src/components/Card.stories.tsx"))
            .unwrap()
            .contains("{ Card as Component }")
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("src/components/Button.stories.tsx")).unwrap(),
        "// existing\n"
    );
}

#[test]
fn nuxt_components_generate_while_pages_are_excluded_and_existing_stories_preserved() {
    let p = TestProject::new();
    p.add_with_contents("components/Button.vue", "<template><button /></template>");
    p.add_with_contents("components/Button.stories.ts", "// existing\n");
    p.add_with_contents("components/Card.vue", "<template><article /></template>");
    p.add("pages/index.vue");
    let root = p.root.to_str().unwrap();
    let args = [root, "--framework", "vue", "--ignore", "pages/**"];
    let check = run(&["check", args[0], args[1], args[2], args[3], args[4]]);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("Card.vue"));
    assert!(!String::from_utf8_lossy(&check.stdout).contains("index.vue"));
    let generated = run(&[
        "check",
        args[0],
        args[1],
        args[2],
        args[3],
        args[4],
        "--generate",
    ]);
    assert_eq!(generated.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("components/Card.stories.ts"))
            .unwrap()
            .contains("./Card.vue")
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("components/Button.stories.ts")).unwrap(),
        "// existing\n"
    );
    assert!(!p.root.join("pages/index.stories.ts").exists());
}

#[test]
fn sveltekit_components_generate_with_routes_ignored_and_preserve_existing() {
    let p = TestProject::new();
    p.add_with_contents("src/lib/Button.svelte", "<button />");
    p.add_with_contents("src/lib/Button.stories.ts", "// existing\n");
    p.add_with_contents("src/lib/Card.svelte", "<article />");
    p.add("src/routes/+page.svelte");
    let root = p.root.to_str().unwrap();
    let args = [root, "--framework", "svelte", "--ignore", "src/routes/**"];
    let check = run(&["check", args[0], args[1], args[2], args[3], args[4]]);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("Card.svelte"));
    assert!(!String::from_utf8_lossy(&check.stdout).contains("+page.svelte"));
    let generated = run(&[
        "check",
        args[0],
        args[1],
        args[2],
        args[3],
        args[4],
        "--generate",
    ]);
    assert_eq!(generated.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("src/lib/Card.stories.ts"))
            .unwrap()
            .contains("./Card.svelte")
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("src/lib/Button.stories.ts")).unwrap(),
        "// existing\n"
    );
    assert!(!p.root.join("src/routes/+page.stories.ts").exists());
}

#[test]
fn sveltekit_route_generation_uses_reserved_plus_story_filename() {
    let p = TestProject::new();
    p.add_with_contents("src/routes/+page.svelte", "<main>Page</main>");
    let out = run(&[
        "check",
        p.root.to_str().unwrap(),
        "--framework",
        "svelte",
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("src/routes/+page.stories.ts"))
            .unwrap()
            .contains("./+page.svelte")
    );
}

#[test]
fn analog_generation_excludes_pages_and_preserves_existing_story() {
    let p = TestProject::new();
    p.add("src/app/components/button.component.ts");
    p.add_with_contents("src/app/components/button.stories.ts", "// existing\n");
    p.add_with_contents(
        "src/app/components/card.ts",
        "@Component({}) export class Card {}\n",
    );
    p.add_with_contents(
        "src/app/pages/index.page.ts",
        "@Component({}) export default class HomePage {}\n",
    );
    let root = p.root.to_str().unwrap();
    let args = [
        root,
        "--framework",
        "angular",
        "--ignore",
        "src/app/pages/**",
    ];
    let check = run(&["check", args[0], args[1], args[2], args[3], args[4]]);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("card.ts"));
    assert!(!String::from_utf8_lossy(&check.stdout).contains("index.page.ts"));
    let out = run(&[
        "check",
        args[0],
        args[1],
        args[2],
        args[3],
        args[4],
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("src/app/components/card.stories.ts"))
            .unwrap()
            .contains("{ Card as Component }")
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("src/app/components/button.stories.ts")).unwrap(),
        "// existing\n"
    );
    assert!(!p.root.join("src/app/pages/index.page.stories.ts").exists());
}

#[test]
fn analog_default_export_page_uses_a_default_import() {
    let p = TestProject::new();
    p.add_with_contents(
        "src/app/pages/index.page.ts",
        "@Component({}) export default class HomePage {}\n",
    );
    let out = run(&[
        "check",
        p.root.to_str().unwrap(),
        "--framework",
        "angular",
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("src/app/pages/index.page.stories.ts"))
            .unwrap()
            .contains("import Component from './index.page';")
    );
}

#[test]
fn web_components_cli_reports_generates_and_preserves_stories() {
    let p = TestProject::new();
    p.add_with_contents("button.js", "customElements.define('my-button', Button);\n");
    p.add_with_contents("button.stories.ts", "// existing\n");
    p.add_with_contents("card.ts", "@customElement('my-card') class Card {}\n");
    p.add_with_contents(
        "ignored.js",
        "customElements.define('my-ignored', Ignored);\n",
    );
    let root = p.root.to_str().unwrap();
    let args = [
        root,
        "--framework",
        "web-components",
        "--ignore",
        "ignored.js",
    ];
    let out = run(&["check", args[0], args[1], args[2], args[3], args[4]]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("card.ts"));
    for command in ["coverage", "report"] {
        let out = run(&[command, args[0], args[1], args[2], args[3], args[4]]);
        assert_eq!(out.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&out.stdout).contains("50.0% (1/2 components)"));
    }
    let out = run(&[
        "check",
        args[0],
        args[1],
        args[2],
        args[3],
        args[4],
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("card.stories.ts"))
            .unwrap()
            .contains("document.createElement('my-card')")
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("button.stories.ts")).unwrap(),
        "// existing\n"
    );
}

#[test]
fn web_components_cli_refuses_existing_target() {
    let p = TestProject::new();
    p.add_with_contents("card.js", "customElements.define('my-card', Card);\n");
    std::fs::create_dir(p.root.join("card.stories.js")).unwrap();
    let out = run(&[
        "check",
        p.root.to_str().unwrap(),
        "--framework",
        "web-components",
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("already exists"));
}

#[test]
fn html_cli_reports_and_generates_only_explicit_fragments() {
    let p = TestProject::new();
    p.add_with_contents("button.component.html", "<button>Button</button>");
    p.add_with_contents("button.component.stories.ts", "// preserve\n");
    p.add_with_contents("card.component.html", "<p>Card</p>");
    p.add("page.html");
    p.add("ignored.component.html");
    let root = p.root.to_str().unwrap();
    let args = [
        root,
        "--framework",
        "html",
        "--ignore",
        "ignored.component.html",
    ];
    let out = run(&["check", args[0], args[1], args[2], args[3], args[4]]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("card.component.html"));
    for command in ["coverage", "report"] {
        let out = run(&[command, args[0], args[1], args[2], args[3], args[4]]);
        assert_eq!(out.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&out.stdout).contains("50.0% (1/2 components)"));
    }
    let out = run(&[
        "check",
        args[0],
        args[1],
        args[2],
        args[3],
        args[4],
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(p.root.join("card.component.stories.js").exists());
    assert_eq!(
        std::fs::read_to_string(p.root.join("button.component.stories.ts")).unwrap(),
        "// preserve\n"
    );
    assert!(!p.root.join("page.stories.js").exists());
}

#[test]
fn html_cli_refuses_existing_story_target() {
    let p = TestProject::new();
    p.add("card.component.html");
    std::fs::create_dir(p.root.join("card.component.stories.js")).unwrap();
    let out = run(&[
        "check",
        p.root.to_str().unwrap(),
        "--framework",
        "html",
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("already exists"));
}

#[test]
fn ember_cli_reports_and_generates_colocated_components() {
    let p = TestProject::new();
    p.add("app/components/button.gjs");
    p.add_with_contents("app/components/button.stories.js", "// existing\n");
    p.add("app/components/card.hbs");
    p.add("app/components/card.js");
    p.add("app/components/ignored.gts");
    let root = p.root.to_str().unwrap();
    let args = [
        root,
        "--framework",
        "ember",
        "--ignore",
        "app/components/ignored.gts",
    ];
    let out = run(&["check", args[0], args[1], args[2], args[3], args[4]]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("card.hbs"));
    for command in ["coverage", "report"] {
        let out = run(&[command, args[0], args[1], args[2], args[3], args[4]]);
        assert_eq!(out.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&out.stdout).contains("50.0% (1/2 components)"));
    }
    let out = run(&[
        "check",
        args[0],
        args[1],
        args[2],
        args[3],
        args[4],
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(p.root.join("app/components/card.stories.js"))
            .unwrap()
            .contains("<Card />")
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("app/components/button.stories.js")).unwrap(),
        "// existing\n"
    );
}

#[test]
fn ember_cli_refuses_existing_target() {
    let p = TestProject::new();
    p.add("app/components/card.gjs");
    std::fs::create_dir(p.root.join("app/components/card.stories.js")).unwrap();
    let out = run(&[
        "check",
        p.root.to_str().unwrap(),
        "--framework",
        "ember",
        "--generate",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("already exists"));
}
