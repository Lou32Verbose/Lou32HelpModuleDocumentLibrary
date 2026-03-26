use crate::pages::{
    render_document_page, render_home_page, render_search_page, render_tag_page, render_tags_page,
    render_topic_page, render_topics_page,
};
use anyhow::{Context, Result};
use lou32help_core::{
    Workspace, WorkspaceView, build_browser_search_index, slug_to_output_path,
    validate_output_relative_path,
};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Summary statistics from a site build.
#[derive(Debug)]
pub struct BuildReport {
    /// Total number of HTML pages written.
    pub page_count: usize,
    /// Number of topic pages.
    pub topic_count: usize,
    /// Number of tag pages.
    pub tag_count: usize,
}

/// Build the static site from a workspace, optionally including drafts.
pub fn build_site(
    workspace: &Workspace,
    include_drafts: bool,
    out_dir: impl AsRef<Path>,
) -> Result<BuildReport> {
    let view = workspace.view(include_drafts);
    build_site_from_view(&view, out_dir)
}

/// Build the static site from a pre-computed workspace view.
pub fn build_site_from_view(
    view: &WorkspaceView<'_>,
    out_dir: impl AsRef<Path>,
) -> Result<BuildReport> {
    let out_dir = out_dir.as_ref();
    if out_dir.exists() {
        fs::remove_dir_all(out_dir)
            .with_context(|| format!("failed to clear {}", out_dir.display()))?;
    }
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let assets_dir = out_dir.join(&view.config().paths.assets_dir);
    fs::create_dir_all(&assets_dir)
        .with_context(|| format!("failed to create {}", assets_dir.display()))?;

    fs::write(assets_dir.join("styles.css"), styles_css())?;
    debug!("wrote styles.css");
    fs::write(assets_dir.join("theme.js"), theme_js())?;
    debug!("wrote theme.js");
    fs::write(
        assets_dir.join("search.js"),
        search_app_js(
            &view.config().search.wasm_module,
            view.config().search.min_query_length,
            view.config().search.max_results,
        ),
    )?;
    debug!("wrote search.js");
    let browser_index = build_browser_search_index(view.documents().iter().copied());
    let search_index_bytes = serde_json::to_vec_pretty(&browser_index)?;
    fs::write(assets_dir.join("search-index.json"), &search_index_bytes)?;
    debug!(
        entries = browser_index.entries.len(),
        bytes = search_index_bytes.len(),
        "wrote search-index.json"
    );

    let mut page_count = 0usize;

    write_page(
        &safe_output_path(out_dir, Path::new("index.html"))?,
        "/",
        render_home_page(view).into_string(),
    )?;
    page_count += 1;

    write_page(
        &safe_output_path(out_dir, Path::new("search/index.html"))?,
        "/search/",
        render_search_page(view).into_string(),
    )?;
    page_count += 1;

    write_page(
        &safe_output_path(out_dir, Path::new("topics/index.html"))?,
        "/topics/",
        render_topics_page(view).into_string(),
    )?;
    page_count += 1;

    write_page(
        &safe_output_path(out_dir, Path::new("tags/index.html"))?,
        "/tags/",
        render_tags_page(view).into_string(),
    )?;
    page_count += 1;

    for topic in view.topic_nodes().values() {
        let page_path = format!("/topics/{}/", topic.path);
        let path = safe_output_path(
            out_dir,
            &PathBuf::from("topics").join(&topic.path).join("index.html"),
        )?;
        write_page(
            &path,
            &page_path,
            render_topic_page(view, topic).into_string(),
        )?;
        debug!(topic = %topic.path, "wrote topic page");
        page_count += 1;
    }

    for tag in view.tag_index().keys() {
        let page_path = format!("/tags/{tag}/");
        let path = safe_output_path(out_dir, &PathBuf::from("tags").join(tag).join("index.html"))?;
        write_page(&path, &page_path, render_tag_page(view, tag).into_string())?;
        debug!(tag = %tag, "wrote tag page");
        page_count += 1;
    }

    for doc in view.documents() {
        let path = safe_output_path(out_dir, &slug_to_output_path(&doc.metadata.slug))?;
        write_page(
            &path,
            &doc.metadata.slug,
            render_document_page(view, doc).into_string(),
        )?;
        debug!(slug = %doc.metadata.slug, "wrote document page");
        page_count += 1;
    }

    Ok(BuildReport {
        page_count,
        topic_count: view.topic_nodes().len(),
        tag_count: view.tag_index().len(),
    })
}

fn write_page(path: &Path, page_path: &str, contents: String) -> Result<()> {
    write_html(path, finalize_html(page_path, contents))
}

fn write_html(path: &Path, contents: String) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

fn safe_output_path(out_dir: &Path, relative: &Path) -> Result<PathBuf> {
    validate_output_relative_path(relative)
        .map_err(|issue| anyhow::anyhow!(issue.message))
        .with_context(|| format!("unsafe site output path '{}'", relative.display()))?;
    Ok(out_dir.join(relative))
}

pub(crate) fn finalize_html(page_path: &str, contents: String) -> String {
    rewrite_root_relative_urls(page_path, &contents)
}

/// Replace root-relative `href="/..."` and `src="/..."` URLs in rendered HTML
/// with relative paths. This enables the static site to be served from any
/// subdirectory or opened directly from the filesystem. Scans raw HTML strings
/// rather than parsing a DOM tree — safe here because the HTML is generated by
/// our own `maud` templates, not user input.
fn rewrite_root_relative_urls(page_path: &str, contents: &str) -> String {
    let mut output = String::with_capacity(contents.len());
    let mut rest = contents;

    loop {
        let href_index = rest.find(r#"href="/"#);
        let src_index = rest.find(r#"src="/"#);
        let next = match (href_index, src_index) {
            (Some(href), Some(src)) if href <= src => Some((href, r#"href=""#.len())),
            (Some(_href), Some(src)) => Some((src, r#"src=""#.len())),
            (Some(href), None) => Some((href, r#"href=""#.len())),
            (None, Some(src)) => Some((src, r#"src=""#.len())),
            (None, None) => None,
        };

        let Some((index, prefix_len)) = next else {
            output.push_str(rest);
            break;
        };

        let value_start = index + prefix_len;
        output.push_str(&rest[..value_start]);

        let value_tail = &rest[value_start..];
        let Some(value_end) = value_tail.find('"') else {
            output.push_str(value_tail);
            break;
        };

        let value = &value_tail[..value_end];
        output.push_str(&relative_site_href(page_path, value));
        rest = &value_tail[value_end..];
    }

    output
}

fn relative_site_href(page_path: &str, target: &str) -> String {
    let (target_path, suffix) = split_url_suffix(target);
    let current_file = site_path_to_output_file(page_path);
    let target_file = site_path_to_output_file(target_path);
    format!(
        "{}{}",
        relative_path_between(&current_file, &target_file),
        suffix
    )
}

fn split_url_suffix(value: &str) -> (&str, &str) {
    let query_index = value.find('?');
    let fragment_index = value.find('#');
    let split_index = match (query_index, fragment_index) {
        (Some(query), Some(fragment)) => query.min(fragment),
        (Some(query), None) => query,
        (None, Some(fragment)) => fragment,
        (None, None) => value.len(),
    };
    value.split_at(split_index)
}

/// Map a site path (e.g. "/topics/powershell/" or "/assets/styles.css") to its
/// output file path. Paths without a file extension get "index.html" appended.
fn site_path_to_output_file(site_path: &str) -> String {
    let trimmed = site_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return "index.html".to_string();
    }

    let normalized = trimmed.trim_end_matches('/');
    let last_segment = normalized.rsplit('/').next().unwrap_or_default();
    if last_segment.contains('.') {
        normalized.to_string()
    } else {
        format!("{normalized}/index.html")
    }
}

/// Compute a relative path from one output file to another by walking up to
/// their common ancestor directory. For example, from "search/index.html" to
/// "assets/styles.css" produces "../assets/styles.css".
fn relative_path_between(from_file: &str, to_file: &str) -> String {
    let from_segments = path_segments(from_file);
    let to_segments = path_segments(to_file);
    let from_dir = &from_segments[..from_segments.len().saturating_sub(1)];

    let mut common = 0usize;
    while common < from_dir.len()
        && common < to_segments.len()
        && from_dir[common] == to_segments[common]
    {
        common += 1;
    }

    let mut parts = Vec::new();
    for _ in common..from_dir.len() {
        parts.push("..".to_string());
    }
    for segment in &to_segments[common..] {
        parts.push((*segment).to_string());
    }

    if parts.is_empty() {
        to_segments
            .last()
            .copied()
            .unwrap_or("index.html")
            .to_string()
    } else {
        parts.join("/")
    }
}

fn path_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn styles_css() -> &'static str {
    include_str!("assets/styles.css")
}

fn theme_js() -> &'static str {
    include_str!("assets/theme.js")
}

fn search_app_js(module_name: &str, min_query_length: usize, max_results: usize) -> String {
    include_str!("assets/search.js.tpl")
        .replace("__WASM_MODULE__", module_name)
        .replace("__MIN_QUERY_LENGTH__", &min_query_length.to_string())
        .replace("__MAX_RESULTS__", &max_results.to_string())
}
