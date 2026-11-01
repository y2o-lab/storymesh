mod support;

use std::{fs, path::Path};
use storymesh::{Framework, ScanOptions, generate_story_skeletons, scan, scan_with_options};
use support::TestProject;

#[test]
fn html_scans_only_explicit_fragments_and_matches_stories() {
    let p = TestProject::new();
    p.add_with_contents("src/button.component.html", "<button>Button</button>");
    p.add("src/__stories__/button.component.stories.ts");
    p.add("src/card.component.html");
    p.add("src/ignored.component.html");
    p.add("src/page.html");
    p.add("src/card.test.component.html");
    p.add("src/.component.html");
    p.add("dist/hidden.component.html");
    let options = ScanOptions {
        ignore_patterns: vec!["src/ignored.component.html".into()],
        ..Default::default()
    };
    let report = scan_with_options(&p.root, Framework::Html, &options).expect("scan");
    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [Path::new("src/card.component.html")]
    );
}

#[test]
fn html_generates_vite_raw_stories_and_preserves_existing() {
    let p = TestProject::new();
    p.add_with_contents("button.component.html", "<button>Button</button>");
    p.add_with_contents("button.component.stories.ts", "// existing\n");
    p.add_with_contents("card.component.html", "<article>Card</article>");
    let report = scan(&p.root, Framework::Html).expect("scan");
    generate_story_skeletons(&p.root, &report).expect("generate");
    let source = fs::read_to_string(p.root.join("card.component.stories.js")).unwrap();
    assert!(source.contains("import markup from './card.component.html?raw'"));
    assert!(source.contains("render: () => markup"));
    assert_eq!(
        fs::read_to_string(p.root.join("button.component.stories.ts")).unwrap(),
        "// existing\n"
    );
    assert_eq!(scan(&p.root, Framework::Html).unwrap().covered_count(), 2);
}
