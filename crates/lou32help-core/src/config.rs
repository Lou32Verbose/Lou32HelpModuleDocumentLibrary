use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Top-level configuration loaded from `lou32help.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lou32HelpConfig {
    /// Site metadata (title, tagline, etc.).
    pub site: SiteConfig,
    /// Filesystem paths for content, output, and assets.
    pub paths: PathConfig,
    /// Search tuning parameters.
    pub search: SearchConfig,
    /// Registered top-level topics.
    #[serde(default)]
    pub topics: Vec<TopicConfig>,
}

/// Metadata about the generated site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// Display title of the site.
    pub title: String,
    /// Short tagline shown on the home page.
    pub tagline: String,
    /// Longer description used in meta tags.
    pub description: String,
    /// Canonical base URL for the site.
    pub base_url: String,
    /// Copyright notice.
    pub copyright: String,
    /// Optional source repository URL (e.g. `https://github.com/owner/repo`).
    /// When set, document pages render an "edit on github" link in the footer.
    #[serde(default)]
    pub source_repo: Option<String>,
    /// Branch to link to when building "edit on github" URLs. Defaults to `master`.
    #[serde(default = "default_source_branch")]
    pub source_branch: String,
}

fn default_source_branch() -> String {
    "master".to_string()
}

/// Filesystem path configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    /// Directory containing Markdown source documents.
    pub content_dir: PathBuf,
    /// Directory where the static site is written.
    pub site_dir: PathBuf,
    /// Subdirectory within `site_dir` for static assets.
    pub assets_dir: PathBuf,
}

/// Configuration for search behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Minimum number of characters before a search query is executed.
    pub min_query_length: usize,
    /// Maximum number of search results returned.
    pub max_results: usize,
    /// Maximum number of related documents shown per page.
    pub related_limit: usize,
    /// Maximum number of featured documents on the home page.
    pub featured_limit: usize,
    /// Name of the WASM module for browser-side search.
    pub wasm_module: String,
}

/// A registered top-level topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    /// Unique key used in topic paths (e.g. `"powershell"`).
    pub key: String,
    /// Human-readable title.
    pub title: String,
    /// Short description of the topic.
    pub description: String,
    /// Sort order for display.
    pub order: usize,
}

impl Lou32HelpConfig {
    /// Load configuration from `lou32help.toml` in the given root directory.
    pub fn load_from(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let path = root.join("lou32help.toml");
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config at {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
    }

    /// Resolve the absolute content directory path.
    pub fn content_dir(&self, root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(&self.paths.content_dir)
    }

    /// Resolve the absolute site output directory path.
    pub fn site_dir(&self, root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(&self.paths.site_dir)
    }

    /// Resolve the absolute assets directory path.
    pub fn assets_dir(&self, root: impl AsRef<Path>) -> PathBuf {
        self.site_dir(root).join(&self.paths.assets_dir)
    }

    /// Build a lookup map from topic key to topic config.
    pub fn topic_registry(&self) -> HashMap<&str, &TopicConfig> {
        self.topics
            .iter()
            .map(|topic| (topic.key.as_str(), topic))
            .collect()
    }

    /// Find a registered topic by its key.
    pub fn topic_by_key(&self, key: &str) -> Option<&TopicConfig> {
        self.topics.iter().find(|topic| topic.key == key)
    }

    /// Derive a human-readable title for a topic path.
    pub fn topic_title(&self, path: &str) -> String {
        let Some(first) = path.split('/').next() else {
            return String::new();
        };

        if path == first {
            if let Some(topic) = self.topic_by_key(first) {
                return topic.title.clone();
            }
        }

        path.rsplit('/')
            .next()
            .map(|segment| {
                segment
                    .split('-')
                    .map(|part| {
                        let mut chars = part.chars();
                        match chars.next() {
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                            None => String::new(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_config_file() {
        let err = Lou32HelpConfig::load_from("/nonexistent/path/that/does/not/exist").unwrap_err();
        assert!(
            format!("{err:#}").contains("failed to read config"),
            "expected missing file error, got: {err:#}"
        );
    }

    #[test]
    fn rejects_malformed_toml() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("lou32help.toml"), "not [ valid toml !!!").unwrap();
        let err = Lou32HelpConfig::load_from(temp.path()).unwrap_err();
        assert!(
            format!("{err:#}").contains("failed to parse"),
            "expected parse error, got: {err:#}"
        );
    }

    #[test]
    fn rejects_missing_required_fields() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("lou32help.toml"),
            "[paths]\ncontent_dir = \"c\"\nsite_dir = \"s\"\nassets_dir = \"a\"\n",
        )
        .unwrap();
        let err = Lou32HelpConfig::load_from(temp.path()).unwrap_err();
        assert!(
            format!("{err:#}").contains("failed to parse"),
            "expected missing field error, got: {err:#}"
        );
    }
}
