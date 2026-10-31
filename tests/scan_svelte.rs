mod support;

use std::path::Path;

use storymesh::{Framework, scan};

use support::TestProject;

#[test]
fn finds_svelte_components_and_excludes_stories_tests_and_generated_files() {
    let project = TestProject::new();
    project.add("src/Button.svelte");
    project.add("src/Button.stories.svelte");
    project.add("src/Card.svelte");
    project.add("src/lowercase.svelte");
    for excluded in [
        "src/Card.test.svelte",
        "src/Card.spec.svelte",
        "src/Card.story.svelte",
        "src/Unmatched.stories.svelte",
        "src/helpers.ts",
        ".svelte-kit/generated/Root.svelte",
        "node_modules/library/External.svelte",
    ] {
        project.add(excluded);
    }

    let report = scan(&project.root, Framework::Svelte).expect("the project should scan");

    assert_eq!(report.framework, Framework::Svelte);
    assert_eq!(report.components.len(), 3);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [
            Path::new("src/Card.svelte"),
            Path::new("src/lowercase.svelte")
        ]
    );
}

#[test]
fn matches_svelte_stories_in_supported_layouts_and_extensions() {
    for extension in ["js", "jsx", "mjs", "cjs", "ts", "tsx", "svelte"] {
        for directory in ["", "stories/", "__stories__/"] {
            for component in ["src/Button/Button.svelte", "src/Button/index.svelte"] {
                let project = TestProject::new();
                project.add(component);
                project.add(&format!("src/Button/{directory}Button.stories.{extension}"));

                let report = scan(&project.root, Framework::Svelte).expect("project scans");

                assert_eq!(report.components.len(), 1);
                assert_eq!(
                    report.covered_count(),
                    1,
                    "{component}: {directory}{extension}"
                );
            }
        }
    }
}

#[test]
fn svelte_stories_match_only_the_corresponding_component_and_respect_ignores() {
    let project = TestProject::new();
    project.add("src/Button.svelte");
    project.add("src/other/Button.stories.svelte");
    project.add("src/Card.svelte");
    project.add("src/Card.stories.svelte");
    project.add("src/Generated.svelte");
    project.add_with_contents(
        ".storymeshignore",
        "src/Card.stories.svelte\nsrc/Generated.svelte\n",
    );

    let report = scan(&project.root, Framework::Svelte).expect("project scans");

    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 0);
}

#[test]
fn sveltekit_routes_are_counted_as_svelte_components() {
    let p = TestProject::new();
    p.add("src/lib/Button.svelte");
    p.add("src/lib/Button.stories.ts");
    p.add("src/lib/Card.svelte");
    p.add("src/routes/+page.svelte");
    p.add("src/routes/+layout.svelte");
    p.add("src/routes/+page.test.svelte");
    p.add("src/routes/+page.ts");
    p.add(".svelte-kit/generated/Root.svelte");
    let report = scan(&p.root, Framework::Svelte).expect("scan");
    assert_eq!(report.components.len(), 4);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [
            Path::new("src/lib/Card.svelte"),
            Path::new("src/routes/+layout.svelte"),
            Path::new("src/routes/+page.svelte"),
        ]
    );
}
