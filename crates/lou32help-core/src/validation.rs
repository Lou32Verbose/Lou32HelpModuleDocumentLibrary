use crate::config::Lou32HelpConfig;
use crate::document::{Document, contains_raw_html, normalize_slug, scaffold_sections};
use crate::path_safety::{RoutePathKind, inspect_route_path};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

// ---- Validation issue code catalog ----
//
// Codes emitted by validate_workspace (this module):
//   raw-html              Body contains raw HTML tags (forbidden for security)
//   orphan-topic          Document topic not registered in lou32help.toml
//   slug-topic-mismatch   Slug does not begin with the expected topic path prefix
//   missing-section       Missing a required heading for the document's page type
//   duplicate-slug        More than one document declares the same slug
//   duplicate-alias       More than one document declares the same alias
//   alias-slug-conflict   An alias normalizes to an existing document slug
//   broken-related-link   A related slug does not resolve to any visible document
//
// Codes emitted by inspect_route_path (path_safety module):
//   invalid-slug-path     Slug contains unsafe segments (traversal, reserved names)
//   invalid-topic-path    Topic path contains unsafe segments
//   invalid-related-path  Related slug contains unsafe segments
//   invalid-tag-path      Tag contains unsafe segments
//   invalid-topic-key     Configured topic key contains unsafe segments
//
// Codes emitted by validate_output_relative_path (path_safety module):
//   unsafe-output-path    Generated output path would escape the site directory

/// Controls which documents are included in validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Validate all documents including drafts.
    Workspace,
    /// Validate only published documents.
    PublicBuild,
}

/// Severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Non-blocking issue.
    Warning,
    /// Blocks builds and must be fixed.
    Error,
}

/// A single validation finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity level.
    pub severity: Severity,
    /// Machine-readable issue code (e.g. `"raw-html"`).
    pub code: String,
    /// Human-readable description.
    pub message: String,
    /// Source file path, if applicable.
    pub path: Option<String>,
    /// Document slug, if applicable.
    pub slug: Option<String>,
}

impl ValidationIssue {
    /// Create an error-severity issue.
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        doc: Option<&Document>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            path: doc.map(|doc| doc.source_path.display().to_string()),
            slug: doc.map(|doc| doc.metadata.slug.clone()),
        }
    }

    /// Create a warning-severity issue.
    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        doc: Option<&Document>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            path: doc.map(|doc| doc.source_path.display().to_string()),
            slug: doc.map(|doc| doc.metadata.slug.clone()),
        }
    }

    /// Returns `true` if this issue has error severity.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl Display for ValidationIssue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} [{}] {}", self.severity, self.code, self.message)?;
        if let Some(path) = &self.path {
            write!(f, " ({path})")?;
        }
        Ok(())
    }
}

/// Validate a set of documents against the workspace configuration.
pub fn validate_workspace(
    docs: &[Document],
    config: &Lou32HelpConfig,
    mode: ValidationMode,
) -> Vec<ValidationIssue> {
    let visible_docs = docs
        .iter()
        .filter(|doc| mode == ValidationMode::Workspace || doc.is_published())
        .collect::<Vec<_>>();

    let mut issues = Vec::new();
    let topics = config.topic_registry();
    let mut slug_index: HashMap<&str, Vec<&Document>> = HashMap::new();
    let mut alias_index: HashMap<&str, Vec<&Document>> = HashMap::new();

    for topic in &config.topics {
        for issue in inspect_route_path(&topic.key, RoutePathKind::TopicKey) {
            issues.push(ValidationIssue::error(
                issue.code,
                format!(
                    "configured topic key '{}' is unsafe: {}",
                    topic.key, issue.message
                ),
                None,
            ));
        }
    }

    for doc in &visible_docs {
        for issue in &doc.path_issues {
            issues.push(ValidationIssue::error(
                issue.code,
                issue.message.clone(),
                Some(doc),
            ));
        }

        slug_index
            .entry(doc.metadata.slug.as_str())
            .or_default()
            .push(doc);

        for alias in &doc.metadata.aliases {
            alias_index.entry(alias.as_str()).or_default().push(doc);
        }

        if !topics.contains_key(doc.top_level_topic()) {
            issues.push(ValidationIssue::error(
                "orphan-topic",
                format!(
                    "document topic '{}' is not registered in lou32help.toml",
                    doc.metadata.topic
                ),
                Some(doc),
            ));
        }

        let expected_prefix = format!("/{}/", doc.metadata.topic);
        if !doc.metadata.slug.starts_with(&expected_prefix) {
            issues.push(ValidationIssue::error(
                "slug-topic-mismatch",
                format!(
                    "slug '{}' does not begin with topic path '{}'",
                    doc.metadata.slug, expected_prefix
                ),
                Some(doc),
            ));
        }

        let heading_set = doc
            .headings
            .iter()
            .map(|heading| heading.to_ascii_lowercase())
            .collect::<HashSet<_>>();

        for section in scaffold_sections(doc.metadata.page_type) {
            if !heading_set.contains(&section.to_ascii_lowercase()) {
                issues.push(ValidationIssue::error(
                    "missing-section",
                    format!(
                        "document is missing required '{}' section for {} pages",
                        section, doc.metadata.page_type
                    ),
                    Some(doc),
                ));
            }
        }

        if contains_raw_html(&doc.body) {
            issues.push(ValidationIssue::error(
                "raw-html",
                "document contains raw HTML which is not allowed; use Markdown syntax instead",
                Some(doc),
            ));
        }
    }

    for (slug, matches) in slug_index {
        if matches.len() > 1 {
            for doc in matches {
                issues.push(ValidationIssue::error(
                    "duplicate-slug",
                    format!("slug '{}' is declared by more than one document", slug),
                    Some(doc),
                ));
            }
        }
    }

    let visible_slugs = visible_docs
        .iter()
        .map(|doc| doc.metadata.slug.as_str())
        .collect::<HashSet<_>>();

    for (alias, matches) in alias_index {
        if matches.len() > 1 {
            for doc in &matches {
                issues.push(ValidationIssue::error(
                    "duplicate-alias",
                    format!("alias '{}' is declared by more than one document", alias),
                    Some(doc),
                ));
            }
        }

        if visible_slugs.contains(normalize_slug(alias).as_str()) {
            for doc in matches {
                issues.push(ValidationIssue::error(
                    "alias-slug-conflict",
                    format!("alias '{}' conflicts with an existing document slug", alias),
                    Some(doc),
                ));
            }
        }
    }

    for doc in &visible_docs {
        for related in &doc.metadata.related {
            if !visible_slugs.contains(related.as_str()) {
                issues.push(ValidationIssue::error(
                    "broken-related-link",
                    format!(
                        "related slug '{}' does not resolve inside the active document set",
                        related
                    ),
                    Some(doc),
                ));
            }
        }
    }

    issues.sort_by(|left, right| {
        left.severity
            .cmp(&right.severity)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.message.cmp(&right.message))
    });
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use lou32help_test_fixtures::{
        default_config_toml, write_workspace, write_workspace_with_config,
    };

    #[test]
    fn raw_html_produces_error() {
        let doc_with_html = r#"---
title: HTML Test
slug: /powershell/networking/html-test/
summary: Has raw HTML
topic: powershell/networking
type: recipe
tags: []
aliases: []
platforms: []
related: []
status: published
updated: 2026-03-20
---

## Goal

Goal text.

## Prerequisites

Need PowerShell.

## Steps

1. Do thing.

## Commands

<div>this is raw html</div>

## Verification

Check file exists.

## Related

- Nothing
"#;
        let temp = write_workspace(&[("content/powershell/networking/html.md", doc_with_html)]);
        let workspace = crate::library::Workspace::load(temp.path()).expect("load");
        let issues = workspace.validate(true);
        let raw_html_issue = issues
            .iter()
            .find(|i| i.code == "raw-html")
            .expect("should have raw-html issue");
        assert_eq!(raw_html_issue.severity, Severity::Error);
    }

    #[test]
    fn unsafe_metadata_routes_produce_errors() {
        let doc = r#"---
title: Unsafe Route
slug: /powershell/../unsafe/
summary: Bad route metadata
topic: powershell\unsafe
type: recipe
tags: [con]
aliases: []
platforms: [windows]
related:
  - /../bad/
status: published
updated: 2026-03-20
---

## Goal

Goal text.

## Prerequisites

Need PowerShell.

## Steps

1. Do thing.

## Commands

```powershell
Write-Host "bad"
```

## Verification

Check file exists.

## Related

- Nothing
"#;
        let temp = write_workspace(&[("content/powershell/unsafe.md", doc)]);
        let workspace = crate::library::Workspace::load(temp.path()).expect("load");
        let issues = workspace.validate(true);

        assert!(issues.iter().any(|issue| issue.code == "invalid-slug-path"));
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == "invalid-topic-path")
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == "invalid-related-path")
        );
        assert!(issues.iter().any(|issue| issue.code == "invalid-tag-path"));
    }

    #[test]
    fn unsafe_topic_keys_produce_errors() {
        let config = format!(
            "{}\n[[topics]]\nkey = \"../bad\"\ntitle = \"Bad\"\ndescription = \"Bad\"\norder = 99\n",
            default_config_toml()
        );
        let temp = write_workspace_with_config(
            &config,
            &[(
                "content/powershell/networking/bits.md",
                lou32help_test_fixtures::RECIPE_DOC,
            )],
        );
        let workspace = crate::library::Workspace::load(temp.path()).expect("load");
        let issues = workspace.validate(true);

        assert!(issues.iter().any(|issue| issue.code == "invalid-topic-key"));
    }
}
