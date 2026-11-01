mod support;

use std::{fs, path::PathBuf};

use storymesh::{ComponentCoverage, CoverageReport, Framework, generate_story_skeletons, scan};

use support::TestProject;

#[test]
fn generates_framework_specific_story_skeletons_without_overwriting() {
    let cases = [
        (
            Framework::Qwik,
            "src/button.tsx",
            "export const Button = component$(() => null);\n",
            "src/button.stories.tsx",
            "import { Button as Component } from './button';",
        ),
        (
            Framework::Qwik,
            "src/Card.tsx",
            "export default component$(() => null);\n",
            "src/Card.stories.tsx",
            "import Component from './Card';",
        ),
        (
            Framework::Solid,
            "src/Button.tsx",
            "export default function Button() {}\n",
            "src/Button.stories.tsx",
            "import Component from './Button';",
        ),
        (
            Framework::Solid,
            "src/Card.jsx",
            "export function Card() {}\n",
            "src/Card.stories.jsx",
            "import { Card as Component } from './Card';",
        ),
        (
            Framework::Preact,
            "src/Button.tsx",
            "export default function Button() {}\n",
            "src/Button.stories.tsx",
            "import Component from './Button';",
        ),
        (
            Framework::Preact,
            "src/Card.jsx",
            "export function Card() {}\n",
            "src/Card.stories.jsx",
            "import { Card as Component } from './Card';",
        ),
        (
            Framework::React,
            "src/Button.tsx",
            "export default function Button() {}\n",
            "src/Button.stories.tsx",
            "import Component from './Button';",
        ),
        (
            Framework::React,
            "src/MissingCard.tsx",
            "// export default is only an example\nexport function MissingCard() {}\n",
            "src/MissingCard.stories.tsx",
            "import { MissingCard as Component } from './MissingCard';",
        ),
        (
            Framework::Vue,
            "src/Card.vue",
            "<template><div>Card</div></template>\n",
            "src/Card.stories.ts",
            "import Component from './Card.vue';",
        ),
        (
            Framework::Svelte,
            "src/Badge.svelte",
            "<span>Badge</span>\n",
            "src/Badge.stories.ts",
            "import Component from './Badge.svelte';",
        ),
        (
            Framework::Angular,
            "src/user-card.component.ts",
            "// export default class Example {}\nconst example = 'export default';\nexport class UserCardComponent {}\n",
            "src/user-card.component.stories.ts",
            "import { UserCardComponent as Component } from './user-card.component';",
        ),
        (
            Framework::Angular,
            "src/index.page.ts",
            "@Component({}) export default class HomePage {}\n",
            "src/index.page.stories.ts",
            "import Component from './index.page';",
        ),
    ];

    for (framework, component, source, story, expected_import) in cases {
        let project = TestProject::new();
        project.add_with_contents(component, source);
        let report = scan(&project.root, framework).expect("the project should scan");

        let generated =
            generate_story_skeletons(&project.root, &report).expect("the story should generate");

        assert_eq!(generated, [PathBuf::from(story)]);
        let contents =
            fs::read_to_string(project.root.join(story)).expect("the generated story should read");
        assert!(contents.contains(expected_import));
        assert!(contents.contains("export const Default = {}"));

        let rescanned = scan(&project.root, framework).expect("the project should rescan");
        assert_eq!(rescanned.covered_count(), 1);
        assert!(rescanned.missing().next().is_none());
    }
}

#[test]
fn angular_prefers_the_matching_named_component_over_a_default_export() {
    let project = TestProject::new();
    project.add_with_contents(
        "src/user-card.component.ts",
        "export class UserCardComponent {}\nexport default [];\n",
    );
    let report = scan(&project.root, Framework::Angular).expect("the project should scan");
    generate_story_skeletons(&project.root, &report).expect("the story should generate");
    let story = fs::read_to_string(project.root.join("src/user-card.component.stories.ts"))
        .expect("the story should read");
    assert!(
        story
            .starts_with("import { UserCardComponent as Component } from './user-card.component';")
    );
}

#[test]
fn generation_distinguishes_default_exports_from_named_reexports() {
    for framework in [
        Framework::React,
        Framework::Preact,
        Framework::Solid,
        Framework::Qwik,
        Framework::Angular,
    ] {
        for (exports, default_import) in [
            ("export { default as Other } from './other';", false),
            ("export { default as Other, value } from './other';", false),
            ("export { default } from './other';", true),
            ("export { Other as default } from './other';", true),
            ("export { value, Other as default, } from './other';", true),
            ("export /* comment */ default class {}", true),
        ] {
            let project = TestProject::new();
            let (component, story, import_path, declaration) = if framework == Framework::Angular {
                (
                    "src/index.page.ts",
                    "src/index.page.stories.ts",
                    "./index.page",
                    "@Component({}) class Page {}",
                )
            } else {
                (
                    "src/Card.tsx",
                    "src/Card.stories.tsx",
                    "./Card",
                    "export function Card() {}",
                )
            };
            project.add_with_contents(component, &format!("{exports}\n{declaration}\n"));
            let report = scan(&project.root, framework).expect("the project should scan");
            generate_story_skeletons(&project.root, &report).expect("the story should generate");
            let contents =
                fs::read_to_string(project.root.join(story)).expect("the story should read");
            let expected = if default_import {
                format!("import Component from '{import_path}';")
            } else {
                let name = if framework == Framework::Angular {
                    "IndexPage"
                } else {
                    "Card"
                };
                format!("import {{ {name} as Component }} from '{import_path}';")
            };
            assert!(
                contents.starts_with(&expected),
                "{framework:?}: {exports}\n{contents}"
            );
        }
    }
}

#[test]
fn preact_rejects_unknown_exports_before_writing_any_stories() {
    let project = TestProject::new();
    project.add_with_contents("src/Button.tsx", "export default function Button() {}\n");
    project.add_with_contents("src/Card.tsx", "export const Other = () => null;\n");
    let report = scan(&project.root, Framework::Preact).expect("the project should scan");
    let error = generate_story_skeletons(&project.root, &report).expect_err("unknown export");
    assert!(error.to_string().contains("src/Card.tsx"));
    assert!(error.to_string().contains("write the story manually"));
    assert!(!project.root.join("src/Button.stories.tsx").exists());
}

#[test]
fn solid_rejects_unknown_exports_without_generating_a_placeholder() {
    let project = TestProject::new();
    project.add_with_contents("src/Card.tsx", "export const Other = () => null;\n");
    let report = scan(&project.root, Framework::Solid).expect("scan");
    let error = generate_story_skeletons(&project.root, &report).expect_err("unknown export");
    assert!(error.to_string().contains("write the story manually"));
    assert!(!project.root.join("src/Card.stories.tsx").exists());
}

#[test]
fn jsx_frameworks_generate_stories_for_matching_export_specifiers() {
    for framework in [
        Framework::React,
        Framework::Preact,
        Framework::Solid,
        Framework::Qwik,
    ] {
        for exports in [
            "export { Card };",
            "export { helper, /* component */ Card, };",
            "export { Card as Card };",
            "export { helper as Card };",
        ] {
            let project = TestProject::new();
            project.add_with_contents(
                "Card.tsx",
                &format!("const Card = () => null; const helper = Card; {exports}\n"),
            );
            let report = scan(&project.root, framework).expect("scan");
            let generated = generate_story_skeletons(&project.root, &report).expect("generate");
            assert_eq!(generated, [PathBuf::from("Card.stories.tsx")]);
            let story =
                fs::read_to_string(project.root.join("Card.stories.tsx")).expect("read story");
            assert!(story.starts_with("import { Card as Component } from './Card';"));
        }
    }
}

#[test]
fn jsx_frameworks_reject_unrecognized_aliased_exports() {
    for framework in [Framework::Preact, Framework::Solid, Framework::Qwik] {
        let p = TestProject::new();
        p.add_with_contents(
            "Card.tsx",
            "const Card = wrapper(() => null); export { Card as Widget };\n",
        );
        let report = scan(&p.root, framework).expect("scan");
        let error = generate_story_skeletons(&p.root, &report).expect_err("unrecognized export");
        assert!(error.to_string().contains("write the story manually"));
        assert!(!p.root.join("Card.stories.tsx").exists());
    }
}

#[test]
fn react_platform_suffix_affects_named_export_guess() {
    let p = TestProject::new();
    p.add_with_contents("Button.web.tsx", "export function Button() {}\n");
    let report = scan(&p.root, Framework::React).expect("scan");
    generate_story_skeletons(&p.root, &report).expect("generate");
    let story = fs::read_to_string(p.root.join("Button.web.stories.tsx")).unwrap();
    assert!(story.contains("render: () => null"));
    assert!(!story.contains("import"));
}

#[test]
fn refuses_to_overwrite_existing_story_files() {
    let project = TestProject::new();
    project.add("src/Button.tsx");
    project.add_with_contents("src/Button.stories.tsx", "// keep me\n");
    let report = CoverageReport {
        framework: Framework::React,
        components: vec![ComponentCoverage {
            component: PathBuf::from("src/Button.tsx"),
            story: None,
        }],
    };

    let error = generate_story_skeletons(&project.root, &report)
        .expect_err("an existing story must not be overwritten");

    assert!(error.to_string().contains("already exists"));
    assert_eq!(
        fs::read_to_string(project.root.join("src/Button.stories.tsx"))
            .expect("the existing story should remain readable"),
        "// keep me\n"
    );
}

#[test]
fn generates_a_buildable_placeholder_when_a_react_export_cannot_be_found() {
    let project = TestProject::new();
    project.add_with_contents("src/Sample2.tsx", "");
    let report = scan(&project.root, Framework::React).expect("the project should scan");

    generate_story_skeletons(&project.root, &report).expect("the placeholder should generate");

    let contents = fs::read_to_string(project.root.join("src/Sample2.stories.tsx"))
        .expect("the placeholder should read");
    assert!(!contents.contains("import"));
    assert!(contents.contains("render: () => null"));
}

#[cfg(unix)]
#[test]
fn generated_stories_escape_filename_control_characters() {
    for (framework, extension, source, import_suffix) in [
        (Framework::Html, "component.html", "<p>Hello</p>", "?raw"),
        (
            Framework::WebComponents,
            "js",
            "customElements.define('my-card', Card);",
            "",
        ),
    ] {
        let project = TestProject::new();
        let stem = "card\n\r\t\u{1}\u{7f}\u{2028}\u{2029}'\\";
        let name = format!("{stem}.{extension}");
        project.add_with_contents(&name, source);
        let report = scan(&project.root, framework).expect("scan");
        let paths = generate_story_skeletons(&project.root, &report).expect("generate");
        assert_eq!(paths.len(), 1);
        let story = fs::read_to_string(project.root.join(&paths[0])).unwrap();
        let escaped_stem = r"card\n\r\t\u0001\u007f\u2028\u2029\'\\";
        assert!(story.contains(&format!("'./{escaped_stem}.{extension}{import_suffix}'")));
        assert!(story.contains(&format!("title: 'Components/{escaped_stem}'")));
        assert!(!story.contains(['\r', '\t', '\u{1}', '\u{7f}', '\u{2028}', '\u{2029}']));
    }
}

#[cfg(unix)]
#[test]
fn module_delimiters_in_filenames_fail_before_any_story_is_written() {
    for (framework, extension, source) in [
        (Framework::Html, "component.html", "<p>Hello</p>"),
        (
            Framework::WebComponents,
            "js",
            "customElements.define('my-card', Card);",
        ),
    ] {
        for delimiter in ['?', '#'] {
            let project = TestProject::new();
            project.add_with_contents(&format!("a.{extension}"), source);
            project.add_with_contents(&format!("z{delimiter}card.{extension}"), source);
            let report = scan(&project.root, framework).expect("scan");
            let error =
                generate_story_skeletons(&project.root, &report).expect_err("unsafe import");
            assert!(error.to_string().contains("write the story manually"));
            assert!(
                !project
                    .root
                    .join(if framework == Framework::Html {
                        "a.component.stories.js"
                    } else {
                        "a.stories.js"
                    })
                    .exists()
            );
        }
    }
}
