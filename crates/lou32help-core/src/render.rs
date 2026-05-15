use ammonia::Builder;
use pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};

/// Content Security Policy header value used in generated HTML pages.
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; \
     img-src 'self' data:; object-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'";

/// One entry in a per-page table of contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    /// Heading level — 2 for h2, 3 for h3.
    pub level: u8,
    /// Slug used as the heading `id` and the TOC link's fragment.
    pub slug: String,
    /// Plain-text heading content.
    pub text: String,
}

/// Render Markdown to sanitized HTML.
///
/// Adds slug-based `id` attributes and clickable `#` anchors to h2/h3 headings,
/// and preserves `class="language-..."` on fenced code blocks.
pub fn markdown_to_html(body: &str) -> String {
    markdown_to_html_with_toc(body).0
}

/// Render Markdown to sanitized HTML and also return a flat list of h2/h3
/// entries suitable for building a table of contents.
pub fn markdown_to_html_with_toc(body: &str) -> (String, Vec<TocEntry>) {
    let parser = Parser::new_ext(body, Options::all());
    let events: Vec<Event<'_>> = parser.collect();
    let toc = extract_toc(&events);
    let events = inject_heading_anchors(events);

    let mut output = String::new();
    html::push_html(&mut output, events.into_iter());

    let html = Builder::default()
        .add_tag_attributes("h1", &["id"])
        .add_tag_attributes("h2", &["id"])
        .add_tag_attributes("h3", &["id"])
        .add_tag_attributes("h4", &["id"])
        .add_tag_attributes("h5", &["id"])
        .add_tag_attributes("h6", &["id"])
        .add_tag_attributes("a", &["class"])
        .add_tag_attributes("code", &["class"])
        .clean(&output)
        .to_string();

    (html, toc)
}

/// Walk events and collect every h2/h3 with its slug and plain text.
fn extract_toc(events: &[Event<'_>]) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut i = 0;
    while i < events.len() {
        if let Event::Start(Tag::Heading { level, .. }) = &events[i] {
            let level_n = match level {
                HeadingLevel::H2 => 2u8,
                HeadingLevel::H3 => 3u8,
                _ => {
                    i += 1;
                    continue;
                }
            };
            if let Some(end) = find_heading_end(events, i) {
                let text = collect_text(&events[i + 1..end]);
                let slug = slugify(&text);
                if !slug.is_empty() {
                    entries.push(TocEntry {
                        level: level_n,
                        slug,
                        text,
                    });
                }
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    entries
}

/// Walk the event stream and, for each h2/h3 heading, inject a slug-based id
/// and append a small anchor link before the closing tag.
fn inject_heading_anchors(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut out = Vec::with_capacity(events.len() + 16);
    let mut i = 0;

    while i < events.len() {
        let event = &events[i];

        if let Event::Start(Tag::Heading {
            level,
            id,
            classes,
            attrs,
        }) = event
        {
            if matches!(level, HeadingLevel::H2 | HeadingLevel::H3) {
                let end_idx = match find_heading_end(&events, i) {
                    Some(idx) => idx,
                    None => {
                        out.push(event.clone());
                        i += 1;
                        continue;
                    }
                };

                let text = collect_text(&events[i + 1..end_idx]);
                let slug = slugify(&text);
                let new_id: CowStr<'_> = if id.is_some() {
                    id.clone().unwrap()
                } else {
                    CowStr::Boxed(slug.clone().into_boxed_str())
                };

                out.push(Event::Start(Tag::Heading {
                    level: *level,
                    id: Some(new_id.clone()),
                    classes: classes.clone(),
                    attrs: attrs.clone(),
                }));

                for inner in &events[i + 1..end_idx] {
                    out.push(inner.clone());
                }

                out.push(Event::Html(CowStr::Boxed(
                    format!(" <a class=\"heading-link\" href=\"#{new_id}\">#</a>")
                        .into_boxed_str(),
                )));

                out.push(events[end_idx].clone());
                i = end_idx + 1;
                continue;
            }
        }

        out.push(event.clone());
        i += 1;
    }

    out
}

fn find_heading_end(events: &[Event<'_>], start: usize) -> Option<usize> {
    let level = if let Event::Start(Tag::Heading { level, .. }) = &events[start] {
        *level
    } else {
        return None;
    };

    let mut depth = 0usize;
    for (idx, event) in events.iter().enumerate().skip(start + 1) {
        match event {
            Event::Start(Tag::Heading { level: l, .. }) if *l == level => depth += 1,
            Event::End(TagEnd::Heading(l)) if *l == level => {
                if depth == 0 {
                    return Some(idx);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

fn collect_text(events: &[Event<'_>]) -> String {
    let mut text = String::new();
    for event in events {
        match event {
            Event::Text(t) | Event::Code(t) => text.push_str(t),
            _ => {}
        }
    }
    text
}

/// Lowercase, alphanumeric, hyphen-joined slug suitable for an HTML `id`.
fn slugify(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev_dash = true;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// Render Markdown to plain text suitable for terminal display.
pub fn markdown_to_terminal(body: &str) -> String {
    let mut output = String::new();

    for line in body.lines() {
        let trimmed = line.trim_start();
        let hashes = trimmed.chars().take_while(|ch| *ch == '#').count();
        if hashes > 0 {
            let heading = trimmed[hashes..].trim();
            if !heading.is_empty() {
                output.push_str(&heading.to_uppercase());
                output.push('\n');
                output.push_str(&"-".repeat(heading.len().min(80)));
                output.push_str("\n\n");
                continue;
            }
        }
        output.push_str(line);
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{markdown_to_html, slugify};

    #[test]
    fn sanitizes_script_tags_and_dangerous_links() {
        let rendered = markdown_to_html(
            r#"
<script>alert("x")</script>

<a href="javascript:alert('x')" onclick="alert('x')">bad</a>
"#,
        );

        assert!(!rendered.contains("<script"));
        assert!(!rendered.contains("javascript:"));
        assert!(!rendered.contains("onclick="));
    }

    #[test]
    fn keeps_normal_markdown_output() {
        let rendered = markdown_to_html(
            r#"
## Heading

[Safe link](/docs/)

```powershell
Write-Host "ok"
```
"#,
        );

        assert!(rendered.contains(r#"<h2 id="heading">"#));
        assert!(rendered.contains(r#"class="heading-link""#));
        assert!(rendered.contains(r##"href="#heading""##));
        assert!(rendered.contains(r#"<a href="/docs/""#));
        assert!(rendered.contains(">Safe link</a>"));
        assert!(rendered.contains(r#"<code class="language-powershell">"#));
    }

    #[test]
    fn slugifies_predictably() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  multiple   spaces  "), "multiple-spaces");
        assert_eq!(slugify("Punctuation! And? Symbols."), "punctuation-and-symbols");
        assert_eq!(slugify("Already-Hyphenated"), "already-hyphenated");
        assert_eq!(slugify("Tags & Platforms"), "tags-platforms");
    }

    #[test]
    fn anchors_h3_too() {
        let rendered = markdown_to_html("### A Sub-Heading\n\nBody.\n");
        assert!(rendered.contains(r#"<h3 id="a-sub-heading">"#));
    }
}
