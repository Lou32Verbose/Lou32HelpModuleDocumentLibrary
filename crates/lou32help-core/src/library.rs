use crate::config::{Lou32HelpConfig, TopicConfig};
use crate::document::{Document, normalize_alias, normalize_slug};
use crate::projection;
use crate::search::{SearchIndex, SearchQuery, SearchResult, build_search_index, search_index};
use crate::validation::{ValidationIssue, ValidationMode, validate_workspace};
use anyhow::{Context, Result};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A loaded workspace consisting of configuration and all parsed documents.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Root directory of the workspace.
    pub root: PathBuf,
    /// Parsed configuration from `lou32help.toml`.
    pub config: Lou32HelpConfig,
    /// All documents loaded from the content directory.
    pub documents: Vec<Document>,
}

/// A node in the topic tree.
#[derive(Debug, Clone)]
pub struct TopicNode {
    /// Full topic path (e.g. `powershell/networking`).
    pub path: String,
    /// Human-readable title.
    pub title: String,
    /// Optional description (set for registered top-level topics).
    pub description: Option<String>,
    /// Parent topic path, if any.
    pub parent: Option<String>,
    /// Child topic paths.
    pub children: Vec<String>,
    /// Slugs of documents directly under this topic.
    pub documents: Vec<String>,
    /// Sort order.
    pub order: usize,
}

/// A pre-computed, read-only view of a workspace filtered by visibility.
#[derive(Debug, Clone)]
pub struct WorkspaceView<'a> {
    workspace: &'a Workspace,
    include_drafts: bool,
    documents: Vec<&'a Document>,
    slug_index: HashMap<String, &'a Document>,
    alias_index: HashMap<String, &'a Document>,
    search_index: SearchIndex,
    topic_nodes: BTreeMap<String, TopicNode>,
    topic_documents: HashMap<String, Vec<&'a Document>>,
    tag_index: BTreeMap<String, Vec<&'a Document>>,
    recent_documents: Vec<&'a Document>,
    top_level_topics_with_counts: Vec<(&'a TopicConfig, usize)>,
    platforms: Vec<String>,
}

impl Workspace {
    /// Load the workspace from the given root directory.
    pub fn load(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let config = Lou32HelpConfig::load_from(&root)?;
        let content_dir = config.content_dir(&root);

        let paths: Vec<PathBuf> = WalkDir::new(&content_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("md"))
                    .unwrap_or(false)
            })
            .map(|entry| entry.into_path())
            .collect();

        let mut documents = paths
            .into_iter()
            .par_bridge()
            .map(|path| Document::from_file(&path))
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("failed to load documents from {}", content_dir.display()))?;

        documents.sort_by(|left, right| left.metadata.slug.cmp(&right.metadata.slug));

        Ok(Self {
            root,
            config,
            documents,
        })
    }

    /// Create a pre-computed view of the workspace, optionally including drafts.
    pub fn view(&self, include_drafts: bool) -> WorkspaceView<'_> {
        let documents = projection::visible_documents(&self.documents, include_drafts);
        let slug_index = projection::slug_index(&documents);
        let alias_index = projection::alias_index(&documents);
        let search_index = build_search_index(documents.iter().copied());
        let topic_nodes = projection::topic_nodes(&self.config, &documents);
        let topic_documents = projection::topic_documents(&documents);
        let tag_index = projection::tag_index(&documents);
        let recent_documents = projection::recent_documents(&documents);
        let top_level_topics_with_counts =
            projection::top_level_topics_with_counts(&self.config, &documents);
        let platforms = projection::platforms(&documents);

        WorkspaceView {
            workspace: self,
            include_drafts,
            documents,
            slug_index,
            alias_index,
            search_index,
            topic_nodes,
            topic_documents,
            tag_index,
            recent_documents,
            top_level_topics_with_counts,
            platforms,
        }
    }

    /// Return visible documents, optionally including drafts.
    pub fn visible_documents(&self, include_drafts: bool) -> Vec<&Document> {
        self.view(include_drafts).documents().to_vec()
    }

    /// Run validation checks on the workspace.
    pub fn validate(&self, include_drafts: bool) -> Vec<ValidationIssue> {
        let mode = if include_drafts {
            ValidationMode::Workspace
        } else {
            ValidationMode::PublicBuild
        };
        validate_workspace(&self.documents, &self.config, mode)
    }

    /// Find a document by slug or alias.
    pub fn find_document(&self, slug_or_alias: &str, include_drafts: bool) -> Option<&Document> {
        self.view(include_drafts).find_document(slug_or_alias)
    }

    /// Build and return the search index.
    pub fn search_index(&self, include_drafts: bool) -> SearchIndex {
        self.view(include_drafts).search_index().clone()
    }

    /// Execute a search query against the workspace.
    pub fn search(&self, query: &SearchQuery, include_drafts: bool) -> Vec<SearchResult> {
        self.view(include_drafts).search(query)
    }

    /// Return documents explicitly listed in a document's `related` field.
    pub fn explicit_related<'a>(
        &'a self,
        doc: &'a Document,
        include_drafts: bool,
    ) -> Vec<&'a Document> {
        self.view(include_drafts).explicit_related(doc)
    }

    /// Return documents that link back to the given document.
    pub fn backlinks<'a>(&'a self, doc: &'a Document, include_drafts: bool) -> Vec<&'a Document> {
        self.view(include_drafts).backlinks(doc)
    }

    /// Return algorithmically computed related documents.
    pub fn computed_related<'a>(
        &'a self,
        doc: &'a Document,
        include_drafts: bool,
    ) -> Vec<&'a Document> {
        self.view(include_drafts).computed_related(doc)
    }

    /// Return the topic tree.
    pub fn topic_nodes(&self, include_drafts: bool) -> BTreeMap<String, TopicNode> {
        self.view(include_drafts).topic_nodes().clone()
    }

    /// Return documents under a given topic path.
    pub fn documents_for_topic<'a>(
        &'a self,
        topic_path: &str,
        include_drafts: bool,
    ) -> Vec<&'a Document> {
        self.view(include_drafts).documents_for_topic(topic_path)
    }

    /// Return the tag-to-slug index.
    pub fn tag_index(&self, include_drafts: bool) -> BTreeMap<String, Vec<String>> {
        self.view(include_drafts)
            .tag_index()
            .iter()
            .map(|(tag, docs)| {
                (
                    tag.clone(),
                    docs.iter().map(|doc| doc.metadata.slug.clone()).collect(),
                )
            })
            .collect()
    }

    /// Return the most recently updated documents.
    pub fn recent_documents(&self, include_drafts: bool, limit: usize) -> Vec<&Document> {
        self.view(include_drafts).recent_documents(limit)
    }

    /// Return top-level topics with their document counts.
    pub fn top_level_topics_with_counts(
        &self,
        include_drafts: bool,
    ) -> Vec<(&crate::config::TopicConfig, usize)> {
        self.view(include_drafts)
            .top_level_topics_with_counts()
            .to_vec()
    }
}

impl<'a> WorkspaceView<'a> {
    /// Return a reference to the underlying workspace.
    pub fn workspace(&self) -> &'a Workspace {
        self.workspace
    }

    /// Return the workspace configuration.
    pub fn config(&self) -> &'a Lou32HelpConfig {
        &self.workspace.config
    }

    /// Whether this view includes draft documents.
    pub fn include_drafts(&self) -> bool {
        self.include_drafts
    }

    /// Return the visible documents in this view.
    pub fn documents(&self) -> &[&'a Document] {
        &self.documents
    }

    /// Return all unique platforms across visible documents.
    pub fn platforms(&self) -> &[String] {
        &self.platforms
    }

    /// Find a document by slug or alias within this view.
    pub fn find_document(&self, slug_or_alias: &str) -> Option<&'a Document> {
        let slug = normalize_slug(slug_or_alias);
        if let Some(doc) = self.slug_index.get(&slug) {
            return Some(*doc);
        }

        let alias = normalize_alias(slug_or_alias);
        self.alias_index.get(&alias).copied()
    }

    /// Return the pre-built search index.
    pub fn search_index(&self) -> &SearchIndex {
        &self.search_index
    }

    /// Execute a search query against this view's index.
    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        search_index(&self.search_index, query)
    }

    /// Return documents explicitly listed in a document's `related` field.
    pub fn explicit_related(&self, doc: &'a Document) -> Vec<&'a Document> {
        projection::explicit_related(&self.slug_index, doc)
    }

    /// Return documents that link back to the given document.
    pub fn backlinks(&self, doc: &'a Document) -> Vec<&'a Document> {
        projection::backlinks(&self.documents, doc)
    }

    /// Return algorithmically computed related documents.
    pub fn computed_related(&self, doc: &'a Document) -> Vec<&'a Document> {
        projection::computed_related(
            &self.documents,
            doc,
            self.workspace.config.search.related_limit,
        )
    }

    /// Return the pre-computed topic tree.
    pub fn topic_nodes(&self) -> &BTreeMap<String, TopicNode> {
        &self.topic_nodes
    }

    /// Return documents under a given topic path.
    pub fn documents_for_topic(&self, topic_path: &str) -> Vec<&'a Document> {
        self.topic_documents
            .get(&crate::document::normalize_topic(topic_path))
            .cloned()
            .unwrap_or_default()
    }

    /// Return the tag-to-documents index.
    pub fn tag_index(&self) -> &BTreeMap<String, Vec<&'a Document>> {
        &self.tag_index
    }

    /// Return documents with the given tag.
    pub fn tag_documents(&self, tag: &str) -> Vec<&'a Document> {
        self.tag_index.get(tag).cloned().unwrap_or_default()
    }

    /// Return the most recently updated documents, up to `limit`.
    pub fn recent_documents(&self, limit: usize) -> Vec<&'a Document> {
        let mut recent = self.recent_documents.clone();
        recent.truncate(limit);
        recent
    }

    /// Return top-level topics with their document counts.
    pub fn top_level_topics_with_counts(&self) -> &[(&'a TopicConfig, usize)] {
        &self.top_level_topics_with_counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::PageType;
    use crate::validation::Severity;
    use lou32help_test_fixtures::{RECIPE_DOC, write_workspace};
    use std::fs;

    #[test]
    fn parses_document_and_keeps_slug_stable_across_file_moves() {
        let temp = write_workspace(&[("content/powershell/networking/a.md", RECIPE_DOC)]);
        let workspace = Workspace::load(temp.path()).expect("load workspace");
        assert_eq!(
            workspace.documents[0].metadata.slug,
            "/powershell/networking/bits-transfer/"
        );

        fs::rename(
            temp.path().join("content/powershell/networking/a.md"),
            temp.path().join("content/powershell/networking/moved.md"),
        )
        .expect("rename");

        let moved = Workspace::load(temp.path()).expect("load moved workspace");
        assert_eq!(
            moved.documents[0].metadata.slug,
            "/powershell/networking/bits-transfer/"
        );
    }

    #[test]
    fn search_finds_aliases_and_filters_by_type() {
        let temp = write_workspace(&[("content/powershell/networking/bits.md", RECIPE_DOC)]);
        let workspace = Workspace::load(temp.path()).expect("load workspace");
        let view = workspace.view(true);
        let results = view.search(&SearchQuery {
            query: "start-bitstransfer".to_string(),
            topic: Some("powershell".to_string()),
            page_type: Some(PageType::Recipe),
            platform: Some("windows".to_string()),
            max_results: Some(10),
        });

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "/powershell/networking/bits-transfer/");
    }

    #[test]
    fn public_build_validation_excludes_drafts_and_catches_broken_links() {
        let temp = write_workspace(&[
            ("content/powershell/networking/ok.md", RECIPE_DOC),
            (
                "content/powershell/networking/draft.md",
                r#"---
title: Draft Page
slug: /powershell/networking/draft-page/
summary: Draft page
topic: powershell/networking
type: recipe
tags: [draft]
aliases: [draft-page]
platforms: [windows]
related:
  - /powershell/networking/missing/
status: draft
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
Write-Host "draft"
```

## Verification

Check file exists.

## Related

- Nothing
"#,
            ),
        ]);
        let workspace = Workspace::load(temp.path()).expect("load workspace");
        let workspace_issues = workspace.validate(true);
        assert!(
            workspace_issues
                .iter()
                .any(|issue| issue.code == "broken-related-link")
        );

        let public_issues = workspace.validate(false);
        assert!(
            !public_issues
                .iter()
                .any(|issue| issue.code == "broken-related-link")
        );
        assert!(
            public_issues
                .iter()
                .all(|issue| issue.severity != Severity::Error)
        );
    }

    #[test]
    fn workspace_view_caches_topic_and_tag_views() {
        let temp = write_workspace(&[("content/powershell/networking/bits.md", RECIPE_DOC)]);
        let workspace = Workspace::load(temp.path()).expect("load workspace");
        let view = workspace.view(false);

        assert_eq!(view.documents().len(), 1);
        assert_eq!(
            view.documents_for_topic("powershell").len(),
            1,
            "topic documents should be cached on the view"
        );
        assert_eq!(view.tag_documents("bits").len(), 1);
    }

    #[test]
    fn load_skips_non_markdown_files() {
        let temp = write_workspace(&[("content/powershell/networking/bits.md", RECIPE_DOC)]);
        fs::write(
            temp.path().join("content/powershell/networking/notes.txt"),
            "plain text file that should be ignored",
        )
        .expect("write txt");
        let workspace = Workspace::load(temp.path()).expect("load workspace");
        assert_eq!(workspace.documents.len(), 1);
    }

    #[test]
    fn load_handles_empty_content_dir() {
        let temp = write_workspace(&[]);
        let workspace = Workspace::load(temp.path()).expect("load workspace");
        assert!(workspace.documents.is_empty());
    }
}
