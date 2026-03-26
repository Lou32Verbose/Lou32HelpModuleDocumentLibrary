use lou32help_core::{CONTENT_SECURITY_POLICY, Document, WorkspaceView, title_from_slug_leaf};
use maud::{DOCTYPE, Markup, PreEscaped, html};

pub(crate) fn layout(
    view: &WorkspaceView<'_>,
    title: &str,
    description: &str,
    page_path: &str,
    active_nav: Option<&str>,
    body: Markup,
    include_search_script: bool,
) -> Markup {
    let page_title = if title == view.config().site.title {
        title.to_string()
    } else {
        format!("{title} | {}", view.config().site.title)
    };
    let canonical_url = canonical_url(view.config().site.base_url.as_str(), page_path);

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (page_title) }
                meta name="description" content=(description);
                meta http-equiv="Content-Security-Policy" content=(CONTENT_SECURITY_POLICY);
                link rel="canonical" href=(canonical_url);
                link rel="stylesheet" href="/assets/styles.css";
                script src="/assets/theme.js" {}
            }
            body {
                div.page-shell {
                    header.site-header {
                        a.brand href="/" {
                            span.brand-mark { "L32" }
                            span.brand-text {
                                strong { (view.config().site.title.as_str()) }
                                small { (view.config().site.tagline.as_str()) }
                            }
                        }
                        div.header-right {
                            nav.top-nav {
                                a href="/" class=(nav_class(active_nav, "home")) { "Home" }
                                a href="/search/" class=(nav_class(active_nav, "search")) { "Search" }
                                a href="/topics/" class=(nav_class(active_nav, "topics")) { "Topics" }
                                a href="/tags/" class=(nav_class(active_nav, "tags")) { "Tags" }
                            }
                            select id="theme-picker" class="theme-picker" {
                                option value="" { "Auto" }
                                option value="dark" { "Dark" }
                                option value="enterprise" { "Enterprise 2004" }
                                option value="geocities" { "Geocities" }
                                option value="terminal" { "Terminal" }
                                option value="solarized" { "Solarized" }
                                option value="high-contrast" { "High Contrast" }
                                option value="blueprint" { "Blueprint" }
                                option value="nord" { "Nord" }
                                option value="paper" { "Paper" }
                            }
                        }
                    }
                    main.page-main {
                        (body)
                    }
                    footer.site-footer {
                        p { (view.config().site.description.as_str()) }
                        p { (view.config().site.copyright.as_str()) }
                    }
                }
                @if include_search_script {
                    script type="module" src="/assets/search.js" {}
                }
            }
        }
    }
}

pub(crate) fn document_teaser(doc: &Document) -> Markup {
    html! {
        tr {
            td { a href=(doc.metadata.slug.as_str()) { (doc.metadata.title.as_str()) } }
            td { (doc.metadata.page_type.as_str()) }
            td.muted { (doc.metadata.summary.as_str()) }
        }
    }
}

pub(crate) fn related_column(title: &str, docs: &[&Document]) -> Markup {
    html! {
        div.related-group {
            h3 { (title) }
            @if docs.is_empty() {
                p.muted { "None." }
            } @else {
                ul {
                    @for doc in docs {
                        li {
                            a href=(doc.metadata.slug.as_str()) { (doc.metadata.title.as_str()) }
                            " — "
                            span.muted { (doc.metadata.summary.as_str()) }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) struct Crumb {
    pub label: String,
    pub url: Option<String>,
}

pub(crate) fn topic_breadcrumbs(view: &WorkspaceView<'_>, topic_path: &str) -> Vec<Crumb> {
    let segments = topic_path.split('/').collect::<Vec<_>>();
    let mut crumbs = Vec::new();
    let mut current = String::new();

    for (index, segment) in segments.iter().enumerate() {
        if !current.is_empty() {
            current.push('/');
        }
        current.push_str(segment);

        let label = if index == 0 {
            view.config()
                .topic_by_key(segment)
                .map(|topic| topic.title.clone())
                .unwrap_or_else(|| title_from_slug_leaf(segment))
        } else {
            title_from_slug_leaf(segment)
        };

        crumbs.push(Crumb {
            label,
            url: Some(format!("/topics/{current}/")),
        });
    }

    crumbs
}

pub(crate) fn render_html_body(html_body: &str) -> Markup {
    html! { (PreEscaped(html_body)) }
}

fn nav_class(active: Option<&str>, key: &str) -> &'static str {
    if active == Some(key) { "active" } else { "" }
}

fn canonical_url(base_url: &str, page_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if page_path == "/" {
        format!("{base}/")
    } else {
        format!("{base}{}", page_path)
    }
}
