use crate::document::{Document, PageType, normalize_alias, normalize_slug, normalize_topic};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A search index containing pre-normalized entries for all visible documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    /// Timestamp when the index was generated.
    pub generated_at: String,
    /// Indexed entries, one per document.
    pub entries: Vec<SearchEntry>,
}

/// A browser-optimized search index with bounded body text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSearchIndex {
    /// Timestamp when the index was generated.
    pub generated_at: String,
    /// Indexed entries, one per document.
    pub entries: Vec<BrowserSearchEntry>,
}

/// A single indexed document with pre-normalized search fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntry {
    /// Canonical document slug.
    pub slug: String,
    /// Document title.
    pub title: String,
    /// Document summary.
    pub summary: String,
    /// Topic path.
    pub topic: String,
    /// Page type.
    #[serde(rename = "type")]
    pub page_type: PageType,
    /// Normalized slug for matching.
    pub slug_search: String,
    /// Normalized title for matching.
    pub title_search: String,
    /// Normalized summary for matching.
    pub summary_search: String,
    /// Normalized topic for matching.
    pub topic_search: String,
    /// Normalized tags for matching.
    pub tag_search: Vec<String>,
    /// Normalized aliases for matching.
    pub alias_search: Vec<String>,
    /// Normalized platforms for matching.
    pub platform_search: Vec<String>,
    /// Normalized headings for matching.
    pub heading_search: Vec<String>,
    /// Normalized body text for matching.
    pub body_search: String,
}

/// A browser-oriented indexed document with bounded body text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSearchEntry {
    /// Canonical document slug.
    pub slug: String,
    /// Document title.
    pub title: String,
    /// Document summary.
    pub summary: String,
    /// Topic path.
    pub topic: String,
    /// Page type.
    #[serde(rename = "type")]
    pub page_type: PageType,
    /// Normalized slug for matching.
    pub slug_search: String,
    /// Normalized title for matching.
    pub title_search: String,
    /// Normalized summary for matching.
    pub summary_search: String,
    /// Normalized topic for matching.
    pub topic_search: String,
    /// Normalized tags for matching.
    pub tag_search: Vec<String>,
    /// Normalized aliases for matching.
    pub alias_search: Vec<String>,
    /// Normalized platforms for matching.
    pub platform_search: Vec<String>,
    /// Normalized headings for matching.
    pub heading_search: Vec<String>,
    /// Bounded normalized body text for matching.
    pub body_search: String,
}

/// Parameters for a search query, with optional filters.
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// The search text.
    pub query: String,
    /// Filter to documents under this topic.
    pub topic: Option<String>,
    /// Filter to this page type.
    pub page_type: Option<PageType>,
    /// Filter to this platform.
    pub platform: Option<String>,
    /// Maximum number of results to return.
    pub max_results: Option<usize>,
}

/// A scored search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Canonical document slug.
    pub slug: String,
    /// Document title.
    pub title: String,
    /// Document summary.
    pub summary: String,
    /// Topic path.
    pub topic: String,
    /// Page type.
    #[serde(rename = "type")]
    pub page_type: PageType,
    /// Relevance score (higher is better).
    pub score: i32,
    /// Which fields the query matched on.
    pub matched_on: Vec<String>,
}

impl SearchQuery {
    /// Return the normalized topic filter, if set.
    pub fn normalized_topic(&self) -> Option<String> {
        self.topic.as_ref().map(|topic| normalize_topic(topic))
    }

    /// Return the normalized platform filter, if set.
    pub fn normalized_platform(&self) -> Option<String> {
        self.platform
            .as_ref()
            .map(|platform| normalize_alias(platform))
    }
}

trait SearchEntryLike {
    fn slug(&self) -> &str;
    fn title(&self) -> &str;
    fn summary(&self) -> &str;
    fn topic(&self) -> &str;
    fn page_type(&self) -> PageType;
    fn slug_search(&self) -> &str;
    fn title_search(&self) -> &str;
    fn summary_search(&self) -> &str;
    fn topic_search(&self) -> &str;
    fn tag_search(&self) -> &[String];
    fn alias_search(&self) -> &[String];
    fn platform_search(&self) -> &[String];
    fn heading_search(&self) -> &[String];
    fn body_search(&self) -> &str;
}

impl<T: SearchEntryLike + ?Sized> SearchEntryLike for &T {
    fn slug(&self) -> &str {
        (*self).slug()
    }

    fn title(&self) -> &str {
        (*self).title()
    }

    fn summary(&self) -> &str {
        (*self).summary()
    }

    fn topic(&self) -> &str {
        (*self).topic()
    }

    fn page_type(&self) -> PageType {
        (*self).page_type()
    }

    fn slug_search(&self) -> &str {
        (*self).slug_search()
    }

    fn title_search(&self) -> &str {
        (*self).title_search()
    }

    fn summary_search(&self) -> &str {
        (*self).summary_search()
    }

    fn topic_search(&self) -> &str {
        (*self).topic_search()
    }

    fn tag_search(&self) -> &[String] {
        (*self).tag_search()
    }

    fn alias_search(&self) -> &[String] {
        (*self).alias_search()
    }

    fn platform_search(&self) -> &[String] {
        (*self).platform_search()
    }

    fn heading_search(&self) -> &[String] {
        (*self).heading_search()
    }

    fn body_search(&self) -> &str {
        (*self).body_search()
    }
}

impl SearchEntryLike for SearchEntry {
    fn slug(&self) -> &str {
        &self.slug
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn summary(&self) -> &str {
        &self.summary
    }

    fn topic(&self) -> &str {
        &self.topic
    }

    fn page_type(&self) -> PageType {
        self.page_type
    }

    fn slug_search(&self) -> &str {
        &self.slug_search
    }

    fn title_search(&self) -> &str {
        &self.title_search
    }

    fn summary_search(&self) -> &str {
        &self.summary_search
    }

    fn topic_search(&self) -> &str {
        &self.topic_search
    }

    fn tag_search(&self) -> &[String] {
        &self.tag_search
    }

    fn alias_search(&self) -> &[String] {
        &self.alias_search
    }

    fn platform_search(&self) -> &[String] {
        &self.platform_search
    }

    fn heading_search(&self) -> &[String] {
        &self.heading_search
    }

    fn body_search(&self) -> &str {
        &self.body_search
    }
}

impl SearchEntryLike for BrowserSearchEntry {
    fn slug(&self) -> &str {
        &self.slug
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn summary(&self) -> &str {
        &self.summary
    }

    fn topic(&self) -> &str {
        &self.topic
    }

    fn page_type(&self) -> PageType {
        self.page_type
    }

    fn slug_search(&self) -> &str {
        &self.slug_search
    }

    fn title_search(&self) -> &str {
        &self.title_search
    }

    fn summary_search(&self) -> &str {
        &self.summary_search
    }

    fn topic_search(&self) -> &str {
        &self.topic_search
    }

    fn tag_search(&self) -> &[String] {
        &self.tag_search
    }

    fn alias_search(&self) -> &[String] {
        &self.alias_search
    }

    fn platform_search(&self) -> &[String] {
        &self.platform_search
    }

    fn heading_search(&self) -> &[String] {
        &self.heading_search
    }

    fn body_search(&self) -> &str {
        &self.body_search
    }
}

/// Build a search index from an iterator of documents.
pub fn build_search_index<'a>(docs: impl IntoIterator<Item = &'a Document>) -> SearchIndex {
    let entries = docs
        .into_iter()
        .map(|doc| SearchEntry {
            slug: doc.metadata.slug.clone(),
            title: doc.metadata.title.clone(),
            summary: doc.metadata.summary.clone(),
            topic: doc.metadata.topic.clone(),
            page_type: doc.metadata.page_type,
            slug_search: normalize_slug(&doc.metadata.slug),
            title_search: normalize_text(&doc.metadata.title),
            summary_search: normalize_text(&doc.metadata.summary),
            topic_search: normalize_topic(&doc.metadata.topic),
            tag_search: doc
                .metadata
                .tags
                .iter()
                .map(|value| normalize_alias(value))
                .collect(),
            alias_search: doc
                .metadata
                .aliases
                .iter()
                .map(|value| normalize_alias(value))
                .collect(),
            platform_search: doc
                .metadata
                .platforms
                .iter()
                .map(|value| normalize_alias(value))
                .collect(),
            heading_search: doc
                .headings
                .iter()
                .map(|value| normalize_text(value))
                .collect(),
            body_search: normalize_text(&doc.body),
        })
        .collect();

    SearchIndex {
        generated_at: Utc::now().to_rfc3339(),
        entries,
    }
}

/// Build a browser-optimized search index from an iterator of documents.
pub fn build_browser_search_index<'a>(
    docs: impl IntoIterator<Item = &'a Document>,
) -> BrowserSearchIndex {
    let entries = docs
        .into_iter()
        .map(|doc| BrowserSearchEntry {
            slug: doc.metadata.slug.clone(),
            title: doc.metadata.title.clone(),
            summary: doc.metadata.summary.clone(),
            topic: doc.metadata.topic.clone(),
            page_type: doc.metadata.page_type,
            slug_search: normalize_slug(&doc.metadata.slug),
            title_search: normalize_text(&doc.metadata.title),
            summary_search: normalize_text(&doc.metadata.summary),
            topic_search: normalize_topic(&doc.metadata.topic),
            tag_search: doc
                .metadata
                .tags
                .iter()
                .map(|value| normalize_alias(value))
                .collect(),
            alias_search: doc
                .metadata
                .aliases
                .iter()
                .map(|value| normalize_alias(value))
                .collect(),
            platform_search: doc
                .metadata
                .platforms
                .iter()
                .map(|value| normalize_alias(value))
                .collect(),
            heading_search: doc
                .headings
                .iter()
                .map(|value| normalize_text(value))
                .collect(),
            body_search: bounded_body_search(&doc.body),
        })
        .collect();

    BrowserSearchIndex {
        generated_at: Utc::now().to_rfc3339(),
        entries,
    }
}

/// Search the index with the given query and return scored results.
pub fn search_index(index: &SearchIndex, query: &SearchQuery) -> Vec<SearchResult> {
    search_entries(&index.entries, query)
}

/// Search the browser index with the given query and return scored results.
pub fn search_browser_index(index: &BrowserSearchIndex, query: &SearchQuery) -> Vec<SearchResult> {
    search_entries(&index.entries, query)
}

fn search_entries<E: SearchEntryLike>(entries: &[E], query: &SearchQuery) -> Vec<SearchResult> {
    let normalized_query = normalize_text(&query.query);
    if normalized_query.is_empty() {
        return Vec::new();
    }

    let tokens = split_tokens(&normalized_query);
    let normalized_topic = query.normalized_topic();
    let normalized_platform = query.normalized_platform();

    let mut results = entries
        .iter()
        .filter(|entry| {
            filter_entry(
                entry,
                normalized_topic.as_deref(),
                query.page_type,
                normalized_platform.as_deref(),
            )
        })
        .filter_map(|entry| score_entry(entry, &normalized_query, &tokens))
        .collect::<Vec<_>>();

    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.title.cmp(&right.title))
    });

    let limit = query.max_results.unwrap_or(results.len());
    results.truncate(limit);
    results
}

fn filter_entry(
    entry: &impl SearchEntryLike,
    topic: Option<&str>,
    page_type: Option<PageType>,
    platform: Option<&str>,
) -> bool {
    let topic_matches = topic
        .map(|topic| {
            entry.topic_search() == topic || entry.topic_search().starts_with(&format!("{topic}/"))
        })
        .unwrap_or(true);
    let type_matches = page_type
        .map(|kind| entry.page_type() == kind)
        .unwrap_or(true);
    let platform_matches = platform
        .map(|platform| {
            entry
                .platform_search()
                .iter()
                .any(|value| value == platform)
        })
        .unwrap_or(true);

    topic_matches && type_matches && platform_matches
}

// Scoring tiers (highest to lowest):
//   1. Identity matches — slug, alias, title exact matches indicate the user knows
//      what they want. These dominate ranking to surface canonical lookups instantly.
//   2. Partial identity matches — substring matches on identity fields still carry
//      strong signal but rank below exact matches.
//   3. Metadata matches — summary and topic provide moderate relevance signal.
//   4. Per-token matches — individual query words matched against title words, tags,
//      headings, platforms, and body provide supplementary signal. These accumulate
//      across tokens so multi-word queries reward documents that match broadly.
fn score_entry(
    entry: &impl SearchEntryLike,
    normalized_query: &str,
    tokens: &[String],
) -> Option<SearchResult> {
    let mut score = 0;
    let mut matched = BTreeSet::new();

    // Slug exact: strongest identity signal — user typed the canonical path
    if entry.slug_search() == normalize_slug(normalized_query) {
        score += 240;
        matched.insert("slug".to_string());
    } else if entry.slug_search().contains(normalized_query) {
        // Slug contains: query appears somewhere in the path
        score += 120;
        matched.insert("slug".to_string());
    }

    // Alias exact: registered synonym, nearly as strong as slug
    if entry
        .alias_search()
        .iter()
        .any(|alias| alias == normalized_query)
    {
        score += 220;
        matched.insert("alias".to_string());
    } else if entry
        .alias_search()
        .iter()
        .any(|alias| alias.contains(normalized_query))
    {
        // Alias contains: query is a substring of a registered alias
        score += 110;
        matched.insert("alias".to_string());
    }

    // Title exact: user typed the full display name
    if entry.title_search() == normalized_query {
        score += 200;
        matched.insert("title".to_string());
    } else if entry.title_search().contains(normalized_query) {
        // Title contains: query is a substring of the title
        score += 140;
        matched.insert("title".to_string());
    }

    // Summary contains: the one-line description mentions the query
    if entry.summary_search().contains(normalized_query) {
        score += 80;
        matched.insert("summary".to_string());
    }

    // Topic match: query matches a topic path, giving topical relevance
    if entry.topic_search() == normalized_query
        || entry
            .topic_search()
            .starts_with(&format!("{normalized_query}/"))
    {
        score += 75;
        matched.insert("topic".to_string());
    }

    // Per-token scoring: each query word is matched independently against
    // progressively weaker fields. Scores accumulate across tokens.
    for token in tokens {
        if entry
            .title_search()
            .split_whitespace()
            .any(|word| word == token)
        {
            score += 35; // exact word in title
            matched.insert("title".to_string());
        } else if entry.title_search().contains(token) {
            score += 18; // substring in title
            matched.insert("title".to_string());
        }

        if entry.tag_search().iter().any(|tag| tag == token) {
            score += 28; // exact tag match
            matched.insert("tags".to_string());
        } else if entry.tag_search().iter().any(|tag| tag.contains(token)) {
            score += 15; // substring in tag
            matched.insert("tags".to_string());
        }

        if entry
            .heading_search()
            .iter()
            .any(|heading| heading.contains(token))
        {
            score += 16; // token in section heading
            matched.insert("headings".to_string());
        }

        if entry
            .platform_search()
            .iter()
            .any(|platform| platform == token)
        {
            score += 14; // exact platform match
            matched.insert("platforms".to_string());
        }

        if entry.body_search().contains(token) {
            score += 8; // weakest signal — token somewhere in body text
            matched.insert("body".to_string());
        }
    }

    if score == 0 {
        return None;
    }

    Some(SearchResult {
        slug: entry.slug().to_string(),
        title: entry.title().to_string(),
        summary: entry.summary().to_string(),
        topic: entry.topic().to_string(),
        page_type: entry.page_type(),
        score,
        matched_on: matched.into_iter().collect(),
    })
}

/// Normalize text for search: lowercase, keep only ASCII alphanumeric, collapse whitespace.
pub fn normalize_text(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut last_was_space = true;

    for ch in value.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch);
            last_was_space = false;
        } else if !last_was_space {
            normalized.push(' ');
            last_was_space = true;
        }
    }

    normalized.trim().to_string()
}

fn split_tokens(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Produce a bounded, normalized body excerpt for the browser search index.
/// Limits size so the JSON payload every page load must fetch stays small.
fn bounded_body_search(value: &str) -> String {
    // 48 tokens captures enough leading content for keyword matching.
    const MAX_TOKENS: usize = 48;
    // Hard character ceiling prevents pathologically long tokens (e.g. base64
    // strings or URLs) from inflating the index.
    const MAX_CHARS: usize = 256;

    let normalized = normalize_text(value);
    let mut bounded = normalized
        .split_whitespace()
        .take(MAX_TOKENS)
        .collect::<Vec<_>>()
        .join(" ");
    if bounded.len() > MAX_CHARS {
        bounded.truncate(MAX_CHARS);
        while bounded.ends_with(char::is_whitespace) {
            bounded.pop();
        }
    }
    bounded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_text_strips_non_ascii() {
        assert_eq!(normalize_text("café"), "caf");
        assert_eq!(normalize_text("hello world"), "hello world");
        assert_eq!(normalize_text("test!!!value"), "test value");
    }

    #[test]
    fn search_empty_query_returns_nothing() {
        let index = SearchIndex {
            generated_at: "2026-03-20T00:00:00+00:00".to_string(),
            entries: vec![SearchEntry {
                slug: "/test/".to_string(),
                title: "Test".to_string(),
                summary: "A test".to_string(),
                topic: "test".to_string(),
                page_type: PageType::Reference,
                slug_search: "/test/".to_string(),
                title_search: "test".to_string(),
                summary_search: "a test".to_string(),
                topic_search: "test".to_string(),
                tag_search: vec![],
                alias_search: vec![],
                platform_search: vec![],
                heading_search: vec![],
                body_search: "test body".to_string(),
            }],
        };

        let results = search_index(
            &index,
            &SearchQuery {
                query: "".to_string(),
                ..Default::default()
            },
        );
        assert!(results.is_empty());

        let results = search_index(
            &index,
            &SearchQuery {
                query: "   ".to_string(),
                ..Default::default()
            },
        );
        assert!(results.is_empty());
    }

    #[test]
    fn browser_index_bounds_body_search() {
        let doc = Document {
            source_path: "content/test.md".into(),
            metadata: crate::DocumentMetadata {
                title: "Test".to_string(),
                slug: "/test/".to_string(),
                summary: "Summary".to_string(),
                topic: "powershell".to_string(),
                page_type: PageType::Reference,
                tags: vec!["tag".to_string()],
                aliases: vec!["alias".to_string()],
                platforms: vec!["windows".to_string()],
                related: vec![],
                status: crate::DocStatus::Published,
                updated: chrono::NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(),
            },
            body: "alpha ".repeat(200),
            headings: vec!["Heading".to_string()],
            path_issues: vec![],
        };

        let index = build_browser_search_index([&doc]);
        assert!(index.entries[0].body_search.len() <= 256);
        assert!(index.entries[0].body_search.split_whitespace().count() <= 48);
    }

    #[test]
    fn browser_search_matches_alias_title_and_platform() {
        let index = BrowserSearchIndex {
            generated_at: "2026-03-20T00:00:00+00:00".to_string(),
            entries: vec![BrowserSearchEntry {
                slug: "/powershell/networking/bits-transfer/".to_string(),
                title: "Bits Transfer".to_string(),
                summary: "Download with bits".to_string(),
                topic: "powershell/networking".to_string(),
                page_type: PageType::Recipe,
                slug_search: "/powershell/networking/bits-transfer/".to_string(),
                title_search: "bits transfer".to_string(),
                summary_search: "download with bits".to_string(),
                topic_search: "powershell/networking".to_string(),
                tag_search: vec!["powershell".to_string(), "bits".to_string()],
                alias_search: vec!["start-bitstransfer".to_string()],
                platform_search: vec!["windows".to_string()],
                heading_search: vec!["goal".to_string(), "commands".to_string()],
                body_search: "start bitstransfer transfer files".to_string(),
            }],
        };

        let alias_results = search_browser_index(
            &index,
            &SearchQuery {
                query: "start-bitstransfer".to_string(),
                ..Default::default()
            },
        );
        assert_eq!(alias_results.len(), 1);

        let title_results = search_browser_index(
            &index,
            &SearchQuery {
                query: "bits".to_string(),
                ..Default::default()
            },
        );
        assert_eq!(title_results.len(), 1);

        let platform_results = search_browser_index(
            &index,
            &SearchQuery {
                query: "bits".to_string(),
                platform: Some("windows".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(platform_results.len(), 1);
    }
}
