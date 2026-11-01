mod support;

use std::path::Path;
use storymesh::{Framework, ScanOptions, scan_with_options};
use support::TestProject;

#[test]
fn solid_detects_jsx_candidates_and_respects_story_paths_and_exclusions() {
    let p = TestProject::new();
    for path in [
        "src/Button.tsx",
        "src/__stories__/Button.stories.ts",
        "src/Card.jsx",
        "src/utility.tsx",
        "src/ignored/Hidden.tsx",
        "src/helpers.js",
        "src/main.tsx",
        "src/Card.test.tsx",
        "node_modules/pkg/External.tsx",
    ] {
        p.add(path);
    }
    let options = ScanOptions {
        ignore_patterns: vec!["src/ignored/**".into()],
        ..Default::default()
    };
    let report = scan_with_options(&p.root, Framework::Solid, &options).expect("scan");
    assert_eq!(report.components.len(), 3);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [Path::new("src/Card.jsx"), Path::new("src/utility.tsx")]
    );
}

#[test]
fn solid_same_stem_files_share_story_match() {
    let p = TestProject::new();
    p.add("Button.jsx");
    p.add("Button.tsx");
    p.add("Button.stories.js");
    let report =
        scan_with_options(&p.root, Framework::Solid, &ScanOptions::default()).expect("scan");
    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 2);
}
