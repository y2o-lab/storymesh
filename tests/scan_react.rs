mod support;

use std::path::Path;

use storymesh::{Framework, scan};

use support::TestProject;

#[test]
fn finds_covered_and_missing_react_components() {
    let project = TestProject::new();
    project.add("src/Button.tsx");
    project.add("src/Button.stories.tsx");
    project.add("src/Card.jsx");

    let report = scan(&project.root, Framework::React).expect("the project should scan");

    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [Path::new("src/Card.jsx")]
    );
    assert_eq!(report.percentage(), 50.0);
}

#[test]
fn matches_story_directories_and_index_components() {
    let project = TestProject::new();
    project.add("src/components/Avatar/index.tsx");
    project.add("src/components/Avatar/__stories__/Avatar.stories.ts");
    project.add("src/components/Badge/Badge.tsx");
    project.add("src/components/Badge/stories/Badge.stories.js");

    let report = scan(&project.root, Framework::React).expect("the project should scan");

    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 2);
    assert!(report.missing().next().is_none());
}

#[test]
fn matches_stories_kept_beside_components_in_a_stories_directory() {
    let project = TestProject::new();
    project.add("src/stories/Button.tsx");
    project.add("src/stories/Button.stories.ts");

    let report = scan(&project.root, Framework::React).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.covered_count(), 1);
}

#[test]
fn supports_pascal_case_javascript_components_and_ignores_entrypoints() {
    let project = TestProject::new();
    project.add("src/Alert.js");
    project.add("src/Alert.stories.mjs");
    project.add("src/AlertProps.d.ts");
    project.add("src/helpers.js");
    project.add("src/types.ts");
    project.add("src/entry/main.tsx");
    project.add("src/Main.tsx");
    project.add("src/Main.stories.tsx");

    let report = scan(&project.root, Framework::React).expect("the project should scan");

    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 2);
}

#[test]
fn ignores_generated_dependencies_and_test_files() {
    let project = TestProject::new();
    project.add("node_modules/package/External.tsx");
    project.add("dist/Bundle.jsx");
    project.add("src/Button.test.tsx");
    project.add("src/Button.spec.jsx");
    project.add("src/real-component.tsx");

    let report = scan(&project.root, Framework::React).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [Path::new("src/real-component.tsx")]
    );
}

#[test]
fn empty_projects_have_full_coverage_and_files_are_not_scan_roots() {
    let project = TestProject::new();

    let report = scan(&project.root, Framework::React).expect("the project should scan");
    assert_eq!(report.percentage(), 100.0);

    project.add("src/Button.tsx");
    let error = scan(&project.root.join("src/Button.tsx"), Framework::React)
        .expect_err("a file is not a valid scan root");
    assert!(error.to_string().contains("path is not a directory"));
}

#[test]
fn platform_suffixes_require_matching_story_names() {
    let p = TestProject::new();
    p.add("src/Button.tsx");
    p.add("src/Button.web.tsx");
    p.add("src/Button.ios.tsx");
    p.add("src/Button.native.tsx");
    p.add("src/Button.stories.tsx");
    p.add("src/Button.web.stories.tsx");
    let report = scan(&p.root, Framework::React).expect("scan");
    assert_eq!(report.components.len(), 4);
    assert_eq!(report.covered_count(), 2);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [
            Path::new("src/Button.ios.tsx"),
            Path::new("src/Button.native.tsx")
        ]
    );
}

#[test]
fn next_app_router_files_are_counted_by_filename_not_route_semantics() {
    let p = TestProject::new();
    p.add("app/components/Button.tsx");
    p.add("app/components/Button.stories.tsx");
    p.add("app/components/Card.tsx");
    p.add("app/page.tsx");
    p.add("app/layout.tsx");
    p.add("app/dashboard/page.tsx");
    p.add("app/dashboard/loading.tsx");
    p.add("app/dashboard/route.ts");
    p.add("app/components/Card.test.tsx");
    let report = scan(&p.root, Framework::React).expect("scan");
    assert_eq!(report.components.len(), 6);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [
            Path::new("app/components/Card.tsx"),
            Path::new("app/dashboard/loading.tsx"),
            Path::new("app/dashboard/page.tsx"),
            Path::new("app/layout.tsx"),
            Path::new("app/page.tsx"),
        ]
    );
}

#[test]
fn tanstack_file_routes_follow_react_filename_rules() {
    let p = TestProject::new();
    p.add("src/components/Button.tsx");
    p.add("src/components/Button.stories.tsx");
    p.add("src/components/Card.tsx");
    p.add("src/routes/__root.tsx");
    p.add("src/routes/about.tsx");
    p.add("src/routes/index.tsx");
    p.add("src/routes/posts/index.tsx");
    p.add("src/routeTree.gen.ts");
    p.add("src/routes/about.test.tsx");
    let report = scan(&p.root, Framework::React).expect("scan");
    assert_eq!(report.components.len(), 4);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [
            Path::new("src/components/Card.tsx"),
            Path::new("src/routes/__root.tsx"),
            Path::new("src/routes/about.tsx"),
        ]
    );
}
