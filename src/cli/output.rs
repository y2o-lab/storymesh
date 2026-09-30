use std::{
    io::{self, Write},
    path::PathBuf,
};

use storymesh::{CoverageReport, GenerationResult, SpecCheckResult, SpecIssue};

pub(super) fn print_check(report: &CoverageReport, output: &mut dyn Write) -> io::Result<u8> {
    let missing = report.missing().collect::<Vec<_>>();
    if missing.is_empty() {
        writeln!(
            output,
            "All {} {} components have stories.",
            report.components.len(),
            report.framework.name()
        )?;
        return Ok(0);
    }

    writeln!(
        output,
        "Missing stories for {} {} component(s):",
        missing.len(),
        report.framework.name()
    )?;
    for path in missing {
        writeln!(output, "{}", path.display())?;
    }
    Ok(1)
}

pub(super) fn print_coverage(report: &CoverageReport, output: &mut dyn Write) -> io::Result<u8> {
    writeln!(
        output,
        "{} Storybook coverage: {:.1}% ({}/{} components)",
        report.framework.name(),
        report.percentage(),
        report.covered_count(),
        report.components.len()
    )?;
    Ok(0)
}

pub(super) fn print_generated(paths: &[PathBuf], output: &mut dyn Write) -> io::Result<()> {
    if paths.is_empty() {
        return Ok(());
    }

    writeln!(output, "Generated {} story skeleton(s):", paths.len())?;
    for path in paths {
        writeln!(output, "{}", path.display())?;
    }
    Ok(())
}

pub(super) fn print_report(report: &CoverageReport, output: &mut dyn Write) -> io::Result<u8> {
    print_coverage(report, output)?;
    let missing = report.missing().collect::<Vec<_>>();
    writeln!(output, "Missing: {}", missing.len())?;
    for path in missing {
        writeln!(output, "{}", path.display())?;
    }
    Ok(0)
}

pub(super) fn print_spec_check(result: &SpecCheckResult, output: &mut dyn Write) -> io::Result<u8> {
    writeln!(output, "Storymesh UI Spec\n")?;
    if !result.errors.is_empty() {
        writeln!(output, "Missing stories")?;
        for issue in &result.errors {
            if let SpecIssue::MissingStory { component, story } = issue {
                writeln!(output, "  ✗ {component}/{story}")?;
            }
        }
        writeln!(output)?;
    }
    if !result.warnings.is_empty() {
        writeln!(output, "Undeclared stories (WARNING)")?;
        for issue in &result.warnings {
            if let SpecIssue::UndeclaredStory { component, story } = issue {
                writeln!(output, "  ? {component}/{story}")?;
            }
        }
        writeln!(output)?;
    }
    if result.errors.is_empty() && result.warnings.is_empty() {
        writeln!(output, "No issues found.\n")?;
    }
    writeln!(output, "Summary")?;
    writeln!(
        output,
        "  Components checked: {}",
        result.components_checked
    )?;
    writeln!(output, "  Stories checked:    {}", result.stories_checked)?;
    writeln!(output, "  Errors:             {}", result.errors.len())?;
    writeln!(output, "  Warnings:           {}\n", result.warnings.len())?;
    writeln!(output, "{}", if result.valid { "PASS" } else { "FAIL" })?;
    Ok(if result.valid { 0 } else { 1 })
}

pub(super) fn print_spec_generation(
    result: &GenerationResult,
    output: &mut dyn Write,
) -> io::Result<()> {
    for path in &result.created {
        writeln!(output, "Created {}", path.display())?;
    }
    for path in &result.updated {
        writeln!(output, "Updated {}", path.display())?;
    }
    writeln!(
        output,
        "Created {} spec(s) / Updated {} spec(s) / Skipped {}",
        result.created_specs,
        result.updated.len(),
        result.skipped
    )?;
    if result.existing_story_drafts > 0 {
        writeln!(
            output,
            "Existing stories were not changed. Fill the draft from a fresh Storybook index; `storymesh spec import` can bootstrap an empty spec directory."
        )?;
    }
    if result.generated_stories > 0 {
        writeln!(
            output,
            "Generate a fresh Storybook index and run `storymesh spec check` to verify registration."
        )?;
    }
    Ok(())
}
