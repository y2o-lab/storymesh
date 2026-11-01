mod support;

use std::path::Path;

use storymesh::{Framework, ScanOptions, scan_with_options};
use support::TestProject;

#[test]
fn preact_detects_components_and_respects_story_paths_and_exclusions() {
    let project = TestProject::new();
    for path in [
        "src/Button.tsx",
        "src/__stories__/Button.stories.ts",
        "src/Card.jsx",
        "src/Alert.js",
        "src/ignored/Hidden.tsx",
        "src/helpers.js",
        "src/main.tsx",
        "src/Card.test.tsx",
        "node_modules/pkg/External.tsx",
    ] {
        project.add(path);
    }
    let options = ScanOptions {
        ignore_patterns: vec!["src/ignored/**".into()],
        ..Default::default()
    };
    let report = scan_with_options(&project.root, Framework::Preact, &options).expect("scan");
    assert_eq!(report.components.len(), 3);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [Path::new("src/Alert.js"), Path::new("src/Card.jsx")]
    );
}

#[test]
fn preact_same_stem_components_share_a_story_match() {
    let project = TestProject::new();
    project.add("src/Button.jsx");
    project.add("src/Button.tsx");
    project.add("src/Button.stories.js");
    let report =
        scan_with_options(&project.root, Framework::Preact, &ScanOptions::default()).expect("scan");
    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 2);
}
