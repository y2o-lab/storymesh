mod support;

use std::{fs, path::Path};
use storymesh::{Framework, ScanOptions, generate_story_skeletons, scan, scan_with_options};
use support::TestProject;

#[test]
fn ember_finds_colocated_components_without_double_counting_classes() {
    let p = TestProject::new();
    p.add_with_contents("app/components/hello.gjs", "<template>Hello</template>");
    p.add("app/components/hello.stories.js");
    p.add_with_contents("app/components/card.hbs", "<p>Card</p>");
    p.add("app/components/card.js");
    p.add_with_contents(
        "app/components/nested/user-card.gts",
        "<template>Card</template>",
    );
    p.add("app/components/ignored.gjs");
    p.add("app/components/utility.js");
    p.add("app/templates/components/legacy.hbs");
    p.add("app/components/fake.test.gjs");
    p.add("tests/components/fake.gjs");
    let options = ScanOptions {
        ignore_patterns: vec!["app/components/ignored.gjs".into()],
        ..Default::default()
    };
    let report = scan_with_options(&p.root, Framework::Ember, &options).expect("scan");
    assert_eq!(report.components.len(), 3);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [
            Path::new("app/components/card.hbs"),
            Path::new("app/components/nested/user-card.gts")
        ]
    );
}

#[test]
fn ember_generates_resolver_based_stories_with_nested_scan_roots() {
    let p = TestProject::new();
    p.add("app/components/nested/user-card.gts");
    let root = p.root.join("app/components/nested");
    let report = scan(&root, Framework::Ember).expect("scan");
    generate_story_skeletons(&root, &report).expect("generate");
    let story = fs::read_to_string(root.join("user-card.stories.js")).unwrap();
    assert!(story.contains("import { hbs } from 'ember-cli-htmlbars'"));
    assert!(story.contains("<Nested::UserCard />"));
    assert_eq!(scan(&root, Framework::Ember).unwrap().covered_count(), 1);
}

#[test]
fn ember_rejects_ambiguous_sources_and_unresolvable_names() {
    let p = TestProject::new();
    p.add("app/components/card.gjs");
    p.add("app/components/card.hbs");
    let error = scan(&p.root, Framework::Ember).expect_err("ambiguous source");
    assert!(error.to_string().contains("multiple component templates"));
    let options = ScanOptions {
        ignore_patterns: vec!["app/components/card.hbs".into()],
        ..Default::default()
    };
    let report =
        scan_with_options(&p.root, Framework::Ember, &options).expect("excluded duplicate");
    assert_eq!(report.components.len(), 1);
    let other = TestProject::new();
    other.add("app/components/Bad_name.gjs");
    let report = scan(&other.root, Framework::Ember).expect("scan");
    let error = generate_story_skeletons(&other.root, &report).expect_err("unresolvable name");
    assert!(error.to_string().contains("write the story manually"));
    assert!(
        !other
            .root
            .join("app/components/Bad_name.stories.js")
            .exists()
    );
}
