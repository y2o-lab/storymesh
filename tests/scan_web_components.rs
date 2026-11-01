mod support;

use std::{fs, path::Path};
use storymesh::{Framework, ScanOptions, generate_story_skeletons, scan, scan_with_options};
use support::TestProject;

#[test]
fn web_components_detects_registrations_and_excludes_non_components() {
    let p = TestProject::new();
    p.add_with_contents(
        "src/button.js",
        "customElements.define('my-button', Button);\n",
    );
    p.add("src/__stories__/button.stories.ts");
    p.add_with_contents(
        "src/card.ts",
        "@customElement(\"my-card\") export class Card {}\n",
    );
    p.add_with_contents("src/dynamic.js", "customElements.define(name, Dynamic);\n");
    p.add_with_contents(
        "src/ignored.ts",
        "customElements.define('my-ignored', Ignored);\n",
    );
    p.add_with_contents(
        "src/ordinary.js",
        r#"// customElements.define('fake-one', Fake)
/* @customElement('fake-two') */
const text = "customElements.define('fake-three', Fake)";
const template = `@customElement('fake-four')`;
const pattern = /customElements.define\('fake-five'/;
const other = object.customElements.define('fake-six', Fake);
"#,
    );
    p.add_with_contents(
        "src/excluded.test.ts",
        "@customElement('fake-test') class Test {}\n",
    );
    p.add_with_contents(
        "src/decl.d.ts",
        "@customElement('fake-decl') class Decl {}\n",
    );
    let options = ScanOptions {
        ignore_patterns: vec!["src/ignored.ts".into()],
        ..Default::default()
    };
    let report = scan_with_options(&p.root, Framework::WebComponents, &options).expect("scan");
    assert_eq!(report.components.len(), 3);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [Path::new("src/card.ts"), Path::new("src/dynamic.js")]
    );
}

#[test]
fn web_components_generates_registered_element_stories() {
    let p = TestProject::new();
    p.add_with_contents(
        "counter.js",
        "customElements.define('my-counter', wrap(Counter, { option: true }));\n",
    );
    p.add_with_contents(
        "greeting.ts",
        "@customElement('my-greeting') export class Greeting {}\n",
    );
    let report = scan(&p.root, Framework::WebComponents).expect("scan");
    generate_story_skeletons(&p.root, &report).expect("generate");
    for (story, import, tag) in [
        ("counter.stories.js", "./counter.js", "my-counter"),
        ("greeting.stories.ts", "./greeting.ts", "my-greeting"),
    ] {
        let source = fs::read_to_string(p.root.join(story)).unwrap();
        assert!(source.contains(&format!("import '{import}'")));
        assert!(source.contains(&format!("document.createElement('{tag}')")));
    }
    assert_eq!(
        scan(&p.root, Framework::WebComponents)
            .unwrap()
            .covered_count(),
        2
    );
}

#[test]
fn web_components_rejects_dynamic_and_ambiguous_tags_without_writes() {
    for source in [
        "customElements.define(tag, Element);",
        "customElements.define('one-tag', A); customElements.define('two-tag', B);",
        "@customElement('bad') class Bad {}",
        "customElements.define('my-' + name, Element);",
        "customElements.define('my-button', Button, { extends: 'button' });",
    ] {
        let p = TestProject::new();
        p.add_with_contents("element.ts", source);
        let report = scan(&p.root, Framework::WebComponents).expect("scan");
        assert_eq!(report.components.len(), 1);
        let error = generate_story_skeletons(&p.root, &report).expect_err("ambiguous tag");
        assert!(error.to_string().contains("static custom-element tag"));
        assert!(!p.root.join("element.stories.ts").exists());
    }
}

#[test]
fn web_components_same_stem_files_share_a_story_match() {
    let p = TestProject::new();
    p.add_with_contents("card.js", "customElements.define('js-card', Card);\n");
    p.add_with_contents("card.ts", "@customElement('ts-card') class Card {}\n");
    p.add("card.stories.ts");
    let report = scan(&p.root, Framework::WebComponents).expect("scan");
    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 2);
}

#[test]
fn web_components_accepts_trailing_commas_but_rejects_third_arguments() {
    for constructor in [
        "Element",
        "wrap(Element, { option: true })",
        "class extends HTMLElement {}",
    ] {
        let p = TestProject::new();
        p.add_with_contents(
            "element.js",
            &format!("customElements.define('my-element', {constructor}, /* trailing */);"),
        );
        let report = scan(&p.root, Framework::WebComponents).expect("scan");
        generate_story_skeletons(&p.root, &report).expect("trailing comma is supported");
        assert!(p.root.join("element.stories.js").exists());
    }
    let p = TestProject::new();
    p.add_with_contents(
        "element.js",
        "customElements.define('my-element', Element, { extends: 'button' },);",
    );
    let report = scan(&p.root, Framework::WebComponents).expect("scan");
    assert!(generate_story_skeletons(&p.root, &report).is_err());
    assert!(!p.root.join("element.stories.js").exists());
}
