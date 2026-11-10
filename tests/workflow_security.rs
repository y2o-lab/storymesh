use std::{fs, path::Path};

use serde_yaml::Value;

// Repository policy, not a general-purpose Actions security analyzer.
fn credential_findings(workflow: &Value) -> Vec<String> {
    let mut findings = Vec::new();
    let Some(jobs) = workflow["jobs"].as_mapping() else {
        return vec!["workflow has no jobs mapping".into()];
    };
    for (name, job) in jobs {
        let name = name.as_str().unwrap_or("unknown job");
        let Some(steps) = job["steps"].as_sequence() else {
            continue;
        };
        for step in steps {
            if step["uses"]
                .as_str()
                .is_some_and(|uses| uses.starts_with("actions/checkout@"))
                && step["with"]["persist-credentials"].as_bool() != Some(false)
            {
                findings.push(format!(
                    "{name}: checkout must set persist-credentials: false"
                ));
            }
            let command = step["run"].as_str().unwrap_or_default();
            let checks_or_hooks = [
                "mise run verify",
                "cargo ",
                "npm ci",
                "npm run build",
                "git commit",
            ]
            .iter()
            .any(|operation| command.contains(operation));
            if checks_or_hooks {
                for scope in [workflow, job, step] {
                    if let Some(env) = scope["env"].as_mapping() {
                        for key in ["GITHUB_TOKEN", "GH_TOKEN", "NODE_AUTH_TOKEN", "NPM_TOKEN"] {
                            if env.contains_key(Value::String(key.into())) {
                                findings.push(format!("{name}: {key} must not be exposed to verification, builds, installs or commit hooks"));
                            }
                        }
                    }
                }
            }
        }
    }
    findings
}

#[test]
fn workflows_keep_credentials_out_of_validation_and_commit_hooks() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows");
    for entry in fs::read_dir(directory).expect("read workflows") {
        let path = entry.expect("workflow entry").path();
        if !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("yml" | "yaml")
        ) {
            continue;
        }
        let workflow: Value =
            serde_yaml::from_str(&fs::read_to_string(&path).expect("read workflow"))
                .expect("parse workflow YAML");
        let findings = credential_findings(&workflow);
        assert!(
            findings.is_empty(),
            "{}: {}",
            path.display(),
            findings.join("; ")
        );
    }
}

#[test]
fn credential_guard_rejects_persistence_and_inherited_tokens() {
    for source in [
        "jobs: { check: { steps: [{ uses: 'actions/checkout@v7' }] } }",
        "env: { GH_TOKEN: secret }\njobs: { check: { steps: [{ run: 'mise run verify' }] } }",
        "jobs: { check: { env: { GITHUB_TOKEN: secret }, steps: [{ run: 'git commit -m release' }] } }",
        "jobs: { check: { steps: [{ env: { NODE_AUTH_TOKEN: secret }, run: 'npm ci' }] } }",
    ] {
        let workflow = serde_yaml::from_str(source).expect("parse fixture");
        assert!(!credential_findings(&workflow).is_empty(), "{source}");
    }
    let scoped = serde_yaml::from_str("jobs: { release: { steps: [{ uses: 'actions/checkout@v7', with: { persist-credentials: false } }, { run: 'mise run verify' }, { run: 'git push origin HEAD', env: { GITHUB_TOKEN: secret } }] } }").expect("parse scoped push");
    assert!(credential_findings(&scoped).is_empty());
}
