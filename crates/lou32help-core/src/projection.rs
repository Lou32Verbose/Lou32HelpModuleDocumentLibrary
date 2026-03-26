use crate::config::{Lou32HelpConfig, TopicConfig};
use crate::document::Document;
use crate::library::TopicNode;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub fn visible_documents(docs: &[Document], include_drafts: bool) -> Vec<&Document> {
    docs.iter()
        .filter(|doc| include_drafts || doc.is_published())
        .collect()
}

pub fn slug_index<'a>(docs: &[&'a Document]) -> HashMap<String, &'a Document> {
    docs.iter()
        .map(|doc| (doc.metadata.slug.clone(), *doc))
        .collect()
}

pub fn alias_index<'a>(docs: &[&'a Document]) -> HashMap<String, &'a Document> {
    let mut aliases = HashMap::new();
    for doc in docs {
        for alias in &doc.metadata.aliases {
            aliases.insert(alias.clone(), *doc);
        }
    }
    aliases
}

pub fn explicit_related<'a>(
    slug_index: &HashMap<String, &'a Document>,
    doc: &'a Document,
) -> Vec<&'a Document> {
    doc.metadata
        .related
        .iter()
        .filter_map(|slug| slug_index.get(slug).copied())
        .collect()
}

pub fn backlinks<'a>(docs: &[&'a Document], doc: &'a Document) -> Vec<&'a Document> {
    docs.iter()
        .copied()
        .filter(|candidate| {
            candidate.metadata.slug != doc.metadata.slug
                && candidate
                    .metadata
                    .related
                    .iter()
                    .any(|related| related == &doc.metadata.slug)
        })
        .collect()
}

pub fn computed_related<'a>(
    docs: &[&'a Document],
    doc: &'a Document,
    related_limit: usize,
) -> Vec<&'a Document> {
    let explicit = doc.metadata.related.iter().collect::<BTreeSet<_>>();
    let mut candidates = docs
        .iter()
        .copied()
        .filter(|candidate| candidate.metadata.slug != doc.metadata.slug)
        .filter(|candidate| !explicit.contains(&candidate.metadata.slug))
        .map(|candidate| {
            let same_topic = if candidate.metadata.topic == doc.metadata.topic {
                40
            } else if candidate
                .metadata
                .topic
                .starts_with(&format!("{}/", doc.top_level_topic()))
                || doc
                    .metadata
                    .topic
                    .starts_with(&format!("{}/", candidate.top_level_topic()))
            {
                20
            } else {
                0
            };

            let shared_tags = candidate
                .metadata
                .tags
                .iter()
                .filter(|tag| doc.metadata.tags.contains(tag))
                .count() as i32
                * 15;

            let shared_platforms = candidate
                .metadata
                .platforms
                .iter()
                .filter(|platform| doc.metadata.platforms.contains(platform))
                .count() as i32
                * 5;

            (candidate, same_topic + shared_tags + shared_platforms)
        })
        .filter(|(_, score)| *score > 0)
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0.metadata.title.cmp(&right.0.metadata.title))
    });
    candidates.truncate(related_limit);
    candidates.into_iter().map(|(doc, _)| doc).collect()
}

pub fn topic_nodes(config: &Lou32HelpConfig, docs: &[&Document]) -> BTreeMap<String, TopicNode> {
    let mut nodes = BTreeMap::<String, TopicNode>::new();

    for topic in &config.topics {
        nodes.insert(
            topic.key.clone(),
            TopicNode {
                path: topic.key.clone(),
                title: topic.title.clone(),
                description: Some(topic.description.clone()),
                parent: None,
                children: Vec::new(),
                documents: Vec::new(),
                order: topic.order,
            },
        );
    }

    for doc in docs {
        let segments = doc.metadata.topic.split('/').collect::<Vec<_>>();
        let mut current = String::new();

        for (index, segment) in segments.iter().enumerate() {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(segment);

            let parent = if index == 0 {
                None
            } else {
                Some(segments[..index].join("/"))
            };

            let title = if index == 0 {
                config
                    .topic_by_key(segment)
                    .map(|topic| topic.title.clone())
                    .unwrap_or_else(|| config.topic_title(&current))
            } else {
                config.topic_title(&current)
            };

            let description = if index == 0 {
                config
                    .topic_by_key(segment)
                    .map(|topic| topic.description.clone())
            } else {
                None
            };

            let order = if index == 0 {
                config
                    .topic_by_key(segment)
                    .map(|topic| topic.order)
                    .unwrap_or(usize::MAX)
            } else {
                usize::MAX
            };

            nodes.entry(current.clone()).or_insert(TopicNode {
                path: current.clone(),
                title,
                description,
                parent,
                children: Vec::new(),
                documents: Vec::new(),
                order,
            });
        }

        if let Some(node) = nodes.get_mut(&doc.metadata.topic) {
            node.documents.push(doc.metadata.slug.clone());
        }
    }

    let child_pairs = nodes
        .values()
        .filter_map(|node| {
            node.parent
                .as_ref()
                .map(|parent| (parent.clone(), node.path.clone()))
        })
        .collect::<Vec<_>>();

    for (parent, child) in child_pairs {
        if let Some(node) = nodes.get_mut(&parent) {
            node.children.push(child);
        }
    }

    let order_lookup = nodes
        .iter()
        .map(|(path, node)| (path.clone(), node.order))
        .collect::<HashMap<_, _>>();

    for node in nodes.values_mut() {
        node.children.sort_by(|left, right| {
            let left_order = order_lookup.get(left).copied().unwrap_or(usize::MAX);
            let right_order = order_lookup.get(right).copied().unwrap_or(usize::MAX);
            left_order.cmp(&right_order).then_with(|| left.cmp(right))
        });
        node.documents.sort();
    }

    nodes
}

pub fn topic_documents<'a>(docs: &[&'a Document]) -> HashMap<String, Vec<&'a Document>> {
    let mut topics = HashMap::<String, Vec<&'a Document>>::new();

    for doc in docs {
        let mut current = String::new();
        for segment in doc.metadata.topic.split('/') {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(segment);
            topics.entry(current.clone()).or_default().push(*doc);
        }
    }

    for entries in topics.values_mut() {
        entries.sort_by(|left, right| left.metadata.slug.cmp(&right.metadata.slug));
    }

    topics
}

pub fn tag_index<'a>(docs: &[&'a Document]) -> BTreeMap<String, Vec<&'a Document>> {
    let mut tags = BTreeMap::<String, Vec<&'a Document>>::new();
    for doc in docs {
        for tag in &doc.metadata.tags {
            tags.entry(tag.clone()).or_default().push(*doc);
        }
    }
    for entries in tags.values_mut() {
        entries.sort_by(|left, right| left.metadata.slug.cmp(&right.metadata.slug));
    }
    tags
}

pub fn recent_documents<'a>(docs: &[&'a Document]) -> Vec<&'a Document> {
    let mut sorted: Vec<&Document> = docs.to_vec();
    sorted.sort_by(|left, right| {
        right
            .metadata
            .updated
            .cmp(&left.metadata.updated)
            .then_with(|| left.metadata.title.cmp(&right.metadata.title))
    });
    sorted
}

pub fn top_level_topics_with_counts<'a>(
    config: &'a Lou32HelpConfig,
    docs: &[&Document],
) -> Vec<(&'a TopicConfig, usize)> {
    let mut counts = HashMap::<&str, usize>::new();
    for doc in docs {
        *counts.entry(doc.top_level_topic()).or_default() += 1;
    }

    let mut topics = config
        .topics
        .iter()
        .map(|topic| (topic, *counts.get(topic.key.as_str()).unwrap_or(&0)))
        .collect::<Vec<_>>();
    topics.sort_by(|left, right| left.0.order.cmp(&right.0.order));
    topics
}

pub fn platforms(docs: &[&Document]) -> Vec<String> {
    let mut platforms = docs
        .iter()
        .flat_map(|doc| doc.metadata.platforms.iter().cloned())
        .collect::<Vec<_>>();
    platforms.sort();
    platforms.dedup();
    platforms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocStatus, DocumentMetadata, PageType};
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn make_doc(slug: &str, topic: &str, tags: &[&str], status: DocStatus) -> Document {
        make_doc_full(slug, slug, topic, tags, &[], &[], &[], status, "2026-03-20")
    }

    fn make_doc_full(
        slug: &str,
        title: &str,
        topic: &str,
        tags: &[&str],
        aliases: &[&str],
        platforms: &[&str],
        related: &[&str],
        status: DocStatus,
        updated: &str,
    ) -> Document {
        Document {
            source_path: PathBuf::from(format!("{slug}.md")),
            metadata: DocumentMetadata {
                title: title.to_string(),
                slug: slug.to_string(),
                summary: format!("Summary for {title}"),
                topic: topic.to_string(),
                page_type: PageType::Reference,
                tags: tags.iter().map(|s| s.to_string()).collect(),
                aliases: aliases.iter().map(|s| s.to_string()).collect(),
                platforms: platforms.iter().map(|s| s.to_string()).collect(),
                related: related.iter().map(|s| s.to_string()).collect(),
                status,
                updated: NaiveDate::parse_from_str(updated, "%Y-%m-%d").unwrap(),
            },
            body: "Body text.".to_string(),
            headings: vec!["Heading".to_string()],
            path_issues: vec![],
        }
    }

    #[test]
    fn visible_documents_filters_drafts() {
        let published = make_doc("/a/", "topic", &[], DocStatus::Published);
        let draft = make_doc("/b/", "topic", &[], DocStatus::Draft);
        let all = vec![published, draft];

        let no_drafts = visible_documents(&all, false);
        assert_eq!(no_drafts.len(), 1);
        assert_eq!(no_drafts[0].metadata.slug, "/a/");

        let with_drafts = visible_documents(&all, true);
        assert_eq!(with_drafts.len(), 2);
    }

    #[test]
    fn slug_index_maps_correctly() {
        let doc_a = make_doc("/alpha/", "topic", &[], DocStatus::Published);
        let doc_b = make_doc("/beta/", "topic", &[], DocStatus::Published);
        let docs = vec![&doc_a, &doc_b];

        let index = slug_index(&docs);
        assert_eq!(index.len(), 2);
        assert_eq!(index["/alpha/"].metadata.title, "/alpha/");
        assert_eq!(index["/beta/"].metadata.title, "/beta/");
        assert!(index.get("/gamma/").is_none());
    }

    #[test]
    fn alias_index_handles_multiple_aliases() {
        let doc = make_doc_full(
            "/tool/",
            "Tool",
            "topic",
            &[],
            &["alias-one", "alias-two"],
            &[],
            &[],
            DocStatus::Published,
            "2026-03-20",
        );
        let docs = vec![&doc];

        let index = alias_index(&docs);
        assert_eq!(index.len(), 2);
        assert_eq!(index["alias-one"].metadata.slug, "/tool/");
        assert_eq!(index["alias-two"].metadata.slug, "/tool/");
    }

    #[test]
    fn tag_index_groups_and_sorts() {
        let doc_a = make_doc("/a/", "topic", &["networking", "powershell"], DocStatus::Published);
        let doc_b = make_doc("/b/", "topic", &["networking"], DocStatus::Published);
        let doc_c = make_doc("/c/", "topic", &["security"], DocStatus::Published);
        let docs = vec![&doc_a, &doc_b, &doc_c];

        let index = tag_index(&docs);
        assert_eq!(index.len(), 3);
        assert_eq!(index["networking"].len(), 2);
        assert_eq!(index["powershell"].len(), 1);
        assert_eq!(index["security"].len(), 1);
        // Documents within a tag are sorted by slug.
        assert_eq!(index["networking"][0].metadata.slug, "/a/");
        assert_eq!(index["networking"][1].metadata.slug, "/b/");
    }

    #[test]
    fn recent_documents_sorts_by_updated() {
        let old = make_doc_full("/old/", "Old", "topic", &[], &[], &[], &[], DocStatus::Published, "2025-01-01");
        let new = make_doc_full("/new/", "New", "topic", &[], &[], &[], &[], DocStatus::Published, "2026-03-20");
        let mid = make_doc_full("/mid/", "Mid", "topic", &[], &[], &[], &[], DocStatus::Published, "2025-06-15");
        let docs = vec![&old, &new, &mid];

        let recent = recent_documents(&docs);
        assert_eq!(recent[0].metadata.slug, "/new/");
        assert_eq!(recent[1].metadata.slug, "/mid/");
        assert_eq!(recent[2].metadata.slug, "/old/");
    }

    fn test_config() -> Lou32HelpConfig {
        let toml_str = lou32help_test_fixtures::default_config_toml();
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn topic_nodes_builds_hierarchy() {
        let config = test_config();

        let doc = make_doc("/bits/", "powershell/networking", &[], DocStatus::Published);
        let docs = vec![&doc];

        let nodes = topic_nodes(&config, &docs);
        // Should have both "powershell" and "powershell/networking" nodes.
        assert!(nodes.contains_key("powershell"));
        assert!(nodes.contains_key("powershell/networking"));

        let parent = &nodes["powershell"];
        assert_eq!(parent.title, "PowerShell");
        assert!(parent.parent.is_none());
        assert!(parent.children.contains(&"powershell/networking".to_string()));

        let child = &nodes["powershell/networking"];
        assert_eq!(child.parent.as_deref(), Some("powershell"));
        assert!(child.documents.contains(&"/bits/".to_string()));
    }

    #[test]
    fn computed_related_scores_by_topic_and_tags() {
        let doc_a = make_doc_full(
            "/a/", "A", "powershell/networking",
            &["networking", "bits"], &[], &["windows"], &[],
            DocStatus::Published, "2026-03-20",
        );
        let doc_b = make_doc_full(
            "/b/", "B", "powershell/networking",
            &["networking"], &[], &["windows"], &[],
            DocStatus::Published, "2026-03-20",
        );
        let doc_c = make_doc_full(
            "/c/", "C", "cli-tools/wget",
            &["wget"], &[], &["linux"], &[],
            DocStatus::Published, "2026-03-20",
        );
        let docs = vec![&doc_a, &doc_b, &doc_c];

        // For doc_a, doc_b should rank higher (same topic + shared tags + shared platform).
        let related = computed_related(&docs, &doc_a, 10);
        assert!(!related.is_empty());
        assert_eq!(related[0].metadata.slug, "/b/");
        // doc_c has no shared topic or tags with doc_a, so it should not appear.
        assert!(related.iter().all(|d| d.metadata.slug != "/c/"));
    }
}
