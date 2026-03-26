use crate::layout::{document_teaser, layout, related_column, render_html_body, topic_breadcrumbs};
use lou32help_core::{Document, TopicNode, WorkspaceView, markdown_to_html};
use maud::{Markup, html};

pub(crate) fn render_home_page(view: &WorkspaceView<'_>) -> Markup {
    let title = view.config().site.title.clone();
    let recent = view.recent_documents(view.config().search.featured_limit);
    let topics = view.top_level_topics_with_counts();

    layout(
        view,
        &title,
        &view.config().site.description,
        "/",
        Some("home"),
        html! {
            h1 { (view.config().site.title.as_str()) }
            p { (view.config().site.tagline.as_str()) }
            p {
                a href="/search/" { "Search Docs" }
                " | "
                a href="/topics/" { "Browse Topics" }
                " | "
                a href="/tags/" { "Browse Tags" }
            }

            h2 { "Topics" }
            table {
                @for (topic, count) in topics {
                    tr {
                        td { a href=(format!("/topics/{}/", topic.key)) { (topic.title.as_str()) } }
                        td { (format!("{count}")) }
                        td.muted { (topic.description.as_str()) }
                    }
                }
            }

            h2 { "Recent Pages" }
            (doc_table(&recent))
        },
        false,
    )
}

pub(crate) fn render_topics_page(view: &WorkspaceView<'_>) -> Markup {
    let topics = view.top_level_topics_with_counts();

    layout(
        view,
        "Topics",
        "Browse every topic in the LOU32HELP library.",
        "/topics/",
        Some("topics"),
        html! {
            h1 { "Browse Topics" }
            p { "Start from a domain and drill into subtopics, recipes, references, and troubleshooting pages." }

            table {
                tr { th { "Topic" } th { "Pages" } th { "Description" } }
                @for (topic, count) in topics {
                    tr {
                        td { a href=(format!("/topics/{}/", topic.key)) { (topic.title.as_str()) } }
                        td { (format!("{count}")) }
                        td.muted { (topic.description.as_str()) }
                    }
                }
            }
        },
        false,
    )
}

pub(crate) fn render_search_page(view: &WorkspaceView<'_>) -> Markup {
    layout(
        view,
        "Search",
        "Search the LOU32HELP library.",
        "/search/",
        Some("search"),
        html! {
            h1 { "Search" }
            p { "The browser uses the same normalized index and ranking rules as the terminal CLI." }

            section.search-shell {
                form id="search-form" class="search-form" {
                    label class="search-block" for="query" {
                        span { "Query" }
                        input id="query" type="search" name="q" placeholder="Search commands, fixes, tags, or aliases";
                    }
                    div.filter-row {
                        label class="search-block" for="topic-filter" {
                            span { "Topic" }
                            select id="topic-filter" name="topic" {
                                option value="" { "All topics" }
                                @for topic in view.topic_nodes().values() {
                                    option value=(topic.path.as_str()) { (topic.title.as_str()) }
                                }
                            }
                        }
                        label class="search-block" for="type-filter" {
                            span { "Page Type" }
                            select id="type-filter" name="type" {
                                option value="" { "All types" }
                                option value="reference" { "Reference" }
                                option value="recipe" { "Recipe" }
                                option value="troubleshooting" { "Troubleshooting" }
                                option value="template" { "Template" }
                            }
                        }
                        label class="search-block" for="platform-filter" {
                            span { "Platform" }
                            select id="platform-filter" name="platform" {
                                option value="" { "All platforms" }
                                @for platform in view.platforms() {
                                    option value=(platform.as_str()) { (platform) }
                                }
                            }
                        }
                    }
                }
                p id="search-status" class="search-status" {
                    (format!(
                        "Type at least {} character(s) to search the library.",
                        view.config().search.min_query_length
                    ))
                }
                div id="search-results" class="doc-list" {}
            }
        },
        true,
    )
}

pub(crate) fn render_tags_page(view: &WorkspaceView<'_>) -> Markup {
    let tags = view.tag_index();

    layout(
        view,
        "Tags",
        "Browse tags across the LOU32HELP library.",
        "/tags/",
        Some("tags"),
        html! {
            h1 { "Browse Tags" }
            p { "Jump across the library by recurring tools, platforms, and troubleshooting labels." }

            table {
                tr { th { "Tag" } th { "Pages" } }
                @for (tag, docs) in tags {
                    tr {
                        td { a href=(format!("/tags/{tag}/")) { "#" (tag) } }
                        td { (format!("{}", docs.len())) }
                    }
                }
            }
        },
        false,
    )
}

pub(crate) fn render_topic_page(view: &WorkspaceView<'_>, topic: &TopicNode) -> Markup {
    let docs = view.documents_for_topic(&topic.path);
    let child_nodes = topic
        .children
        .iter()
        .filter_map(|path| view.topic_nodes().get(path).cloned())
        .collect::<Vec<_>>();
    let title = format!("Topic: {}", topic.title);
    let page_path = format!("/topics/{}/", topic.path);

    layout(
        view,
        &title,
        topic
            .description
            .as_deref()
            .unwrap_or("Topic landing page."),
        &page_path,
        Some("topics"),
        html! {
            nav.breadcrumbs aria-label="Breadcrumbs" {
                a href="/" { "Home" }
                span.sep { "/" }
                a href="/topics/" { "Topics" }
                @for crumb in topic_breadcrumbs(view, &topic.path) {
                    span.sep { "/" }
                    @if let Some(url) = crumb.url {
                        a href=(url) { (crumb.label) }
                    } @else {
                        span { (crumb.label) }
                    }
                }
            }
            h1 { (topic.title.as_str()) }
            @if let Some(description) = &topic.description {
                p { (description.as_str()) }
            }
            @if !child_nodes.is_empty() {
                h2 { "Subtopics" }
                ul {
                    @for child in child_nodes {
                        li {
                            a href=(format!("/topics/{}/", child.path)) { (child.title) }
                            @if let Some(description) = child.description {
                                " — " span.muted { (description) }
                            }
                        }
                    }
                }
            }
            h2 { "Pages" }
            p.muted { (format!("{} page(s) in or under this topic.", docs.len())) }
            (doc_table(&docs))
        },
        false,
    )
}

pub(crate) fn render_tag_page(view: &WorkspaceView<'_>, tag: &str) -> Markup {
    let docs = view.tag_documents(tag);
    let page_path = format!("/tags/{tag}/");

    layout(
        view,
        &format!("Tag: {tag}"),
        "Tag index",
        &page_path,
        Some("tags"),
        html! {
            nav.breadcrumbs aria-label="Breadcrumbs" {
                a href="/" { "Home" }
                span.sep { "/" }
                a href="/tags/" { "Tags" }
                span.sep { "/" }
                span { (tag) }
            }
            h1 { "#" (tag) }
            p.muted { (format!("{} page(s) tagged with '{}'.", docs.len(), tag)) }
            (doc_table(&docs))
        },
        false,
    )
}

pub(crate) fn render_document_page(view: &WorkspaceView<'_>, doc: &Document) -> Markup {
    let explicit = view.explicit_related(doc);
    let backlinks = view.backlinks(doc);
    let computed = view.computed_related(doc);
    let html_body = markdown_to_html(&doc.body);

    layout(
        view,
        &doc.metadata.title,
        &doc.metadata.summary,
        &doc.metadata.slug,
        Some("docs"),
        html! {
            nav.breadcrumbs aria-label="Breadcrumbs" {
                a href="/" { "Home" }
                span.sep { "/" }
                a href="/topics/" { "Topics" }
                @for crumb in topic_breadcrumbs(view, &doc.metadata.topic) {
                    span.sep { "/" }
                    @if let Some(url) = crumb.url {
                        a href=(url) { (crumb.label) }
                    } @else {
                        span { (crumb.label) }
                    }
                }
                span.sep { "/" }
                span { (doc.metadata.title.as_str()) }
            }
            article.doc-page {
                header.doc-header {
                    p.muted { (doc.metadata.page_type.as_str()) }
                    h1 { (doc.metadata.title.as_str()) }
                    p { (doc.metadata.summary.as_str()) }
                    p.muted {
                        "Slug: " code { (doc.metadata.slug.as_str()) }
                        " | Topic: " a href=(format!("/topics/{}/", doc.metadata.topic)) { (doc.metadata.topic.as_str()) }
                        " | Updated: " (doc.metadata.updated)
                    }
                    @if !doc.metadata.platforms.is_empty() {
                        p.muted {
                            "Platforms: "
                            @for platform in &doc.metadata.platforms {
                                a.platform href=(format!("/search/?platform={platform}")) { (platform) }
                                " "
                            }
                        }
                    }
                    @if !doc.metadata.tags.is_empty() {
                        p.muted {
                            "Tags: "
                            @for tag in &doc.metadata.tags {
                                a.tag href=(format!("/tags/{tag}/")) { (tag) }
                                " "
                            }
                        }
                    }
                }
                div.doc-content {
                    (render_html_body(&html_body))
                }
            }
            h2 { "Related" }
            div.related-list {
                (related_column("Declared", &explicit))
                (related_column("Backlinks", &backlinks))
                (related_column("See Also", &computed))
            }
        },
        false,
    )
}

fn doc_table(docs: &[&Document]) -> Markup {
    html! {
        @if docs.is_empty() {
            p.muted { "No pages." }
        } @else {
            table {
                tr { th { "Title" } th { "Type" } th { "Summary" } }
                @for doc in docs {
                    (document_teaser(doc))
                }
            }
        }
    }
}
