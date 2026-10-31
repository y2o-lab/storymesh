mod support;

use std::path::Path;

use storymesh::{Framework, scan};

use support::TestProject;

#[test]
fn finds_covered_and_missing_vue_components() {
    let project = TestProject::new();
    project.add("src/Button.vue");
    project.add("src/Button.stories.ts");
    project.add("src/Card.vue");
    project.add("src/Card.test.vue");
    project.add("src/helpers.ts");

    let report = scan(&project.root, Framework::Vue).expect("the project should scan");

    assert_eq!(report.framework, Framework::Vue);
    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [Path::new("src/Card.vue")]
    );
}

#[test]
fn matches_vue_story_directories_and_index_components() {
    let project = TestProject::new();
    project.add("src/Avatar/index.vue");
    project.add("src/Avatar/__stories__/Avatar.stories.js");

    let report = scan(&project.root, Framework::Vue).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.covered_count(), 1);
}

#[test]
fn nuxt_pages_layouts_and_components_follow_vue_file_rules() {
    let p = TestProject::new();
    p.add("components/Button.vue");
    p.add("components/Button.stories.ts");
    p.add("components/Card.vue");
    p.add("pages/index.vue");
    p.add("layouts/default.vue");
    p.add("pages/index.test.vue");
    p.add(".nuxt/generated.vue");
    let report = scan(&p.root, Framework::Vue).expect("scan");
    assert_eq!(report.components.len(), 5);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [
            Path::new(".nuxt/generated.vue"),
            Path::new("components/Card.vue"),
            Path::new("layouts/default.vue"),
            Path::new("pages/index.vue"),
        ]
    );
}

#[test]
fn finds_covered_and_missing_angular_components() {
    let project = TestProject::new();
    project.add("src/button/button.component.ts");
    project.add("src/button/button.stories.ts");
    project.add("src/card/card.component.ts");
    project.add("src/card/card.component.spec.ts");
    project.add("src/card/card.service.ts");

    let report = scan(&project.root, Framework::Angular).expect("the project should scan");

    assert_eq!(report.framework, Framework::Angular);
    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [Path::new("src/card/card.component.ts")]
    );
}

#[test]
fn matches_angular_component_named_stories_in_story_directories() {
    let project = TestProject::new();
    project.add("src/banner/banner.component.ts");
    project.add("src/banner/stories/banner.component.stories.ts");

    let report = scan(&project.root, Framework::Angular).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.covered_count(), 1);
}

#[test]
fn detects_suffix_less_angular_components_and_ignores_comments_and_strings() {
    let project = TestProject::new();
    project.add_with_contents(
        "src/app/app.ts",
        "import { Component } from '@angular/core';\n@Component({ selector: 'app-root' })\nexport class App {}\n",
    );
    project.add("src/app/app.stories.ts");
    project.add_with_contents(
        "src/app/app.config.ts",
        "// @Component({ commentedOut: true })\nexport const note = '@Component({ text: true })';\n",
    );

    let report = scan(&project.root, Framework::Angular).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.covered_count(), 1);
}

#[test]
fn analog_pages_and_decorator_aliases_follow_angular_detection_rules() {
    let p = TestProject::new();
    p.add_with_contents(
        "src/app/components/button.component.ts",
        "export class ButtonComponent {}\n",
    );
    p.add("src/app/components/button.stories.ts");
    p.add_with_contents(
        "src/app/components/card.ts",
        "@Component({}) export class Card {}\n",
    );
    p.add_with_contents("src/app/components/alias.ts", "import { Component as NgComponent } from '@angular/core'; @NgComponent({}) export class Alias {}\n");
    p.add_with_contents(
        "src/app/components/alias.component.ts",
        "@NgComponent({}) export class AliasComponent {}\n",
    );
    p.add_with_contents(
        "src/app/pages/index.page.ts",
        "@Component({}) export default class HomePage {}\n",
    );
    p.add_with_contents(
        "src/app/pages/data.ts",
        "// @Component({})\nexport const data = '@Component({})';\n",
    );
    p.add_with_contents(
        "src/app/components/card.spec.ts",
        "@Component({}) class Test {}\n",
    );
    let report = scan(&p.root, Framework::Angular).expect("scan");
    assert_eq!(report.components.len(), 4);
    assert_eq!(report.covered_count(), 1);
    assert_eq!(
        report.missing().collect::<Vec<_>>(),
        [
            Path::new("src/app/components/alias.component.ts"),
            Path::new("src/app/components/card.ts"),
            Path::new("src/app/pages/index.page.ts"),
        ]
    );
}
