use crate::browser_bundle::bundle_browser_search;
use crate::serve::serve_preview;
use anyhow::{Context, Result, anyhow, bail};
use chrono::Local;
use clap::{Parser, Subcommand};
use lou32help_core::{
    Lou32HelpConfig, PageType, SearchQuery, Workspace, markdown_to_terminal, normalize_slug,
    normalize_topic, scaffold_sections, title_from_slug_leaf,
};
use lou32help_site::build_site_from_view;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "lou32help",
    version,
    about = "Terminal and static-site knowledge base tooling"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        default_value = ".",
        help = "Workspace root directory"
    )]
    root: PathBuf,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Scaffold a new document with frontmatter and required sections")]
    New {
        #[arg(help = "Topic path (e.g. powershell/networking)")]
        topic_path: String,
        #[arg(help = "URL-safe leaf name for the new document")]
        slug: String,
        #[arg(
            long = "type",
            help = "Page type: reference, recipe, troubleshooting, or template"
        )]
        page_type: String,
    },
    #[command(about = "Display a single document in the terminal")]
    Show {
        #[arg(help = "Document slug or alias to look up")]
        slug_or_alias: String,
        #[arg(long, help = "Include draft documents")]
        drafts: bool,
    },
    #[command(about = "Full-text search across the knowledge base")]
    Search {
        #[arg(help = "Search terms (multiple words are joined)")]
        query: Vec<String>,
        #[arg(long, help = "Restrict results to a topic path")]
        topic: Option<String>,
        #[arg(long = "type", help = "Restrict results to a page type")]
        page_type: Option<String>,
        #[arg(long, help = "Restrict results to a platform")]
        platform: Option<String>,
        #[arg(long, help = "Include draft documents in results")]
        drafts: bool,
    },
    #[command(about = "List topics or show documents under a topic")]
    Topics {
        #[arg(help = "Topic path to inspect; omit to list all topics")]
        topic_path: Option<String>,
        #[arg(long, help = "Include draft documents")]
        drafts: bool,
    },
    #[command(about = "Show related documents for a given page")]
    Related {
        #[arg(help = "Document slug to find related pages for")]
        slug: String,
        #[arg(long, help = "Include draft documents")]
        drafts: bool,
    },
    #[command(about = "Validate all documents and configuration")]
    Check {
        #[arg(long, help = "Validate only published documents, excluding drafts")]
        published_only: bool,
    },
    #[command(about = "Generate the static HTML site and browser search bundle")]
    Build {
        #[arg(long, help = "Include draft documents in the build")]
        drafts: bool,
    },
    #[command(about = "Build the site and start a local preview server")]
    Serve {
        #[arg(
            long,
            default_value_t = 4000,
            help = "Port number for the preview server"
        )]
        port: u16,
        #[arg(long, help = "Include draft documents in the build")]
        drafts: bool,
    },
}

pub fn main_entry() {
    init_tracing();

    if let Err(error) = run() {
        error!(error = %format!("{error:#}"), "lou32help command failed");
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            topic_path,
            slug,
            page_type,
        } => run_new(&cli.root, &topic_path, &slug, &page_type),
        Commands::Show {
            slug_or_alias,
            drafts,
        } => run_show(&cli.root, &slug_or_alias, drafts),
        Commands::Search {
            query,
            topic,
            page_type,
            platform,
            drafts,
        } => run_search(&cli.root, &query, topic, page_type, platform, drafts),
        Commands::Topics { topic_path, drafts } => {
            run_topics(&cli.root, topic_path.as_deref(), drafts)
        }
        Commands::Related { slug, drafts } => run_related(&cli.root, &slug, drafts),
        Commands::Check { published_only } => run_check(&cli.root, !published_only),
        Commands::Build { drafts } => run_build(&cli.root, drafts),
        Commands::Serve { port, drafts } => run_serve(&cli.root, port, drafts),
    }
}

fn run_show(root: &Path, slug_or_alias: &str, drafts: bool) -> Result<()> {
    let workspace = load_workspace(root)?;
    let view = workspace.view(drafts);
    let Some(doc) = view.find_document(slug_or_alias) else {
        let suggestions = view.search(&SearchQuery {
            query: slug_or_alias.to_string(),
            max_results: Some(5),
            ..Default::default()
        });
        bail!(
            "document '{slug_or_alias}' not found{}",
            if suggestions.is_empty() {
                String::new()
            } else {
                format!(
                    ". Suggestions: {}",
                    suggestions
                        .into_iter()
                        .map(|item| item.slug)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        );
    };

    println!("{}", doc.metadata.title);
    println!("{}", "=".repeat(doc.metadata.title.len()));
    println!("Slug: {}", doc.metadata.slug);
    println!("Topic: {}", doc.metadata.topic);
    println!("Type: {}", doc.metadata.page_type);
    println!("Status: {}", doc.metadata.status);
    println!("Updated: {}", doc.metadata.updated);
    if !doc.metadata.aliases.is_empty() {
        println!("Aliases: {}", doc.metadata.aliases.join(", "));
    }
    if !doc.metadata.tags.is_empty() {
        println!("Tags: {}", doc.metadata.tags.join(", "));
    }
    if !doc.metadata.platforms.is_empty() {
        println!("Platforms: {}", doc.metadata.platforms.join(", "));
    }
    println!("\n{}\n", doc.metadata.summary);
    println!("{}", markdown_to_terminal(&doc.body));
    Ok(())
}

fn run_search(
    root: &Path,
    query: &[String],
    topic: Option<String>,
    page_type: Option<String>,
    platform: Option<String>,
    drafts: bool,
) -> Result<()> {
    if query.is_empty() {
        bail!("provide a query to search");
    }

    let workspace = load_workspace(root)?;
    let view = workspace.view(drafts);
    let raw_query = query.join(" ");
    if raw_query.trim().chars().count() < view.config().search.min_query_length {
        bail!(
            "query must be at least {} character(s)",
            view.config().search.min_query_length
        );
    }

    let page_type = page_type.as_deref().map(parse_page_type).transpose()?;
    let results = view.search(&SearchQuery {
        query: raw_query,
        topic,
        page_type,
        platform,
        max_results: Some(view.config().search.max_results),
    });

    info!(
        result_count = results.len(),
        include_drafts = drafts,
        "search completed"
    );

    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    for (index, result) in results.iter().enumerate() {
        println!(
            "{}. {} [{}]",
            index + 1,
            result.title,
            result.page_type.as_str()
        );
        println!("   {}", result.slug);
        println!("   {}", result.summary);
        println!("   matched on: {}", result.matched_on.join(", "));
    }
    Ok(())
}

fn run_topics(root: &Path, topic_path: Option<&str>, drafts: bool) -> Result<()> {
    let workspace = load_workspace(root)?;
    let view = workspace.view(drafts);

    if let Some(topic_path) = topic_path {
        let normalized = normalize_topic(topic_path);
        let node = view
            .topic_nodes()
            .get(&normalized)
            .ok_or_else(|| anyhow!("topic '{normalized}' not found"))?;

        println!("{}", node.title);
        println!("{}", "=".repeat(node.title.len()));
        if let Some(description) = &node.description {
            println!("{description}");
        }
        if !node.children.is_empty() {
            println!("\nSubtopics:");
            for child in &node.children {
                if let Some(child_node) = view.topic_nodes().get(child) {
                    println!("- {} ({})", child_node.title, child_node.path);
                }
            }
        }
        let docs = view.documents_for_topic(&normalized);
        if !docs.is_empty() {
            println!("\nPages:");
            for doc in docs {
                println!("- {} :: {}", doc.metadata.title, doc.metadata.slug);
            }
        }
        return Ok(());
    }

    for (topic, count) in view.top_level_topics_with_counts() {
        println!("{} ({})", topic.title, count);
        println!("  {}", topic.description);
    }
    Ok(())
}

fn run_related(root: &Path, slug: &str, drafts: bool) -> Result<()> {
    let workspace = load_workspace(root)?;
    let view = workspace.view(drafts);
    let doc = view
        .find_document(slug)
        .ok_or_else(|| anyhow!("document '{slug}' not found"))?;

    println!("{}", doc.metadata.title);
    println!("{}", "=".repeat(doc.metadata.title.len()));

    print_related_group("Declared", view.explicit_related(doc));
    print_related_group("Backlinks", view.backlinks(doc));
    print_related_group("See Also", view.computed_related(doc));

    Ok(())
}

fn run_check(root: &Path, include_drafts: bool) -> Result<()> {
    let workspace = load_workspace(root)?;
    let issues = validate_workspace(&workspace, include_drafts);
    if issues.is_empty() {
        println!("No validation issues found.");
        return Ok(());
    }

    for issue in &issues {
        println!("{issue}");
    }

    if issues.iter().any(|issue| issue.is_error()) {
        bail!(
            "validation failed with {} error(s)",
            issues.iter().filter(|issue| issue.is_error()).count()
        );
    }

    Ok(())
}

fn run_new(root: &Path, topic_path: &str, slug: &str, page_type: &str) -> Result<()> {
    let config = Lou32HelpConfig::load_from(root)?;
    let topic_path = normalize_topic(topic_path);
    let page_type = parse_page_type(page_type)?;
    let top_level = topic_path
        .split('/')
        .next()
        .ok_or_else(|| anyhow!("invalid topic path"))?;

    if config.topic_by_key(top_level).is_none() {
        bail!("top-level topic '{top_level}' is not registered in lou32help.toml");
    }

    let canonical_slug = if slug.starts_with('/') {
        normalize_slug(slug)
    } else {
        normalize_slug(&format!("{topic_path}/{slug}"))
    };

    if !canonical_slug.starts_with(&format!("/{topic_path}/")) {
        bail!(
            "slug '{}' must live under topic path '{}'",
            canonical_slug,
            topic_path
        );
    }

    let leaf = canonical_slug
        .trim_matches('/')
        .rsplit('/')
        .next()
        .ok_or_else(|| anyhow!("failed to derive slug leaf"))?;
    let file_path = config
        .content_dir(root)
        .join(&topic_path)
        .join(format!("{leaf}.md"));

    if file_path.exists() {
        bail!("file already exists at {}", file_path.display());
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let title = title_from_slug_leaf(leaf);
    let today = Local::now().date_naive();
    let mut body = String::new();
    for section in scaffold_sections(page_type) {
        body.push_str(&format!("## {section}\n\n"));
        body.push_str(match *section {
            "Goal" => "Describe the end result of the procedure.\n\n",
            "Prerequisites" => "- List required tools, access, or context.\n\n",
            "Steps" => "1. Step one.\n2. Step two.\n\n",
            "Commands" => "```powershell\n# Add commands here\n```\n\n",
            "Verification" => "- Explain how to confirm success.\n\n",
            "Synopsis" => "Briefly summarize what this page covers.\n\n",
            "Syntax" => "```text\ncommand --flags <value>\n```\n\n",
            "Parameters/Flags" => "Describe the important parameters and flags.\n\n",
            "Examples" => "Add one or more representative examples.\n\n",
            "Symptoms" => "- Describe the visible problem.\n\n",
            "Cause" => "Explain the underlying cause.\n\n",
            "Resolution" => "Document the fix.\n\n",
            "Use Case" => "Describe when this template is useful.\n\n",
            "Template" => "```text\nTemplate content goes here\n```\n\n",
            "Variables" => "- List placeholders and their meaning.\n\n",
            "Related" => "- Link related pages after they exist.\n\n",
            _ => "\n",
        });
    }

    let contents = format!(
        "\
---
title: {title}
slug: {canonical_slug}
summary: TODO: add a one-line summary.
topic: {topic_path}
type: {}
tags: []
aliases: []
platforms: []
related: []
status: draft
updated: {today}
---

{body}",
        page_type.as_str()
    );

    fs::write(&file_path, contents)
        .with_context(|| format!("failed to write {}", file_path.display()))?;

    println!("Created {}", file_path.display());
    println!("Slug: {canonical_slug}");
    Ok(())
}

fn run_build(root: &Path, drafts: bool) -> Result<()> {
    let workspace = load_workspace(root)?;
    let report = build_and_bundle(root, &workspace, drafts)?;
    let out_dir = workspace.config.site_dir(root);

    println!(
        "Built site to {} ({} pages, {} topics, {} tags).",
        out_dir.display(),
        report.page_count,
        report.topic_count,
        report.tag_count
    );
    Ok(())
}

fn run_serve(root: &Path, port: u16, drafts: bool) -> Result<()> {
    let workspace = load_workspace(root)?;
    let report = build_and_bundle(root, &workspace, drafts)?;
    let site_dir = workspace.config.site_dir(root);

    println!(
        "Built site to {} ({} pages, {} topics, {} tags).",
        site_dir.display(),
        report.page_count,
        report.topic_count,
        report.tag_count
    );

    serve_preview(&site_dir, port)
}

fn print_related_group(label: &str, docs: Vec<&lou32help_core::Document>) {
    println!("\n{label}:");
    if docs.is_empty() {
        println!("- none");
        return;
    }

    for doc in docs {
        println!("- {} :: {}", doc.metadata.title, doc.metadata.slug);
    }
}

fn parse_page_type(value: &str) -> Result<PageType> {
    value.parse()
}

fn load_workspace(root: &Path) -> Result<Workspace> {
    let start = Instant::now();
    let workspace = Workspace::load(root)?;
    info!(
        document_count = workspace.documents.len(),
        elapsed_ms = start.elapsed().as_millis(),
        "loaded workspace"
    );
    Ok(workspace)
}

fn validate_workspace(
    workspace: &Workspace,
    include_drafts: bool,
) -> Vec<lou32help_core::ValidationIssue> {
    let start = Instant::now();
    let issues = workspace.validate(include_drafts);
    let warning_count = issues.iter().filter(|issue| !issue.is_error()).count();
    let error_count = issues.iter().filter(|issue| issue.is_error()).count();
    let mut issue_counts = BTreeMap::<&str, usize>::new();
    for issue in &issues {
        *issue_counts.entry(issue.code.as_str()).or_default() += 1;
    }
    info!(
        include_drafts,
        warning_count,
        error_count,
        issue_codes = %issue_counts
            .into_iter()
            .map(|(code, count)| format!("{code}:{count}"))
            .collect::<Vec<_>>()
            .join(","),
        elapsed_ms = start.elapsed().as_millis(),
        "validated workspace"
    );
    issues
}

fn build_and_bundle(
    root: &Path,
    workspace: &Workspace,
    drafts: bool,
) -> Result<lou32help_site::BuildReport> {
    let issues = validate_workspace(workspace, drafts);
    for issue in &issues {
        println!("{issue}");
    }
    if issues.iter().any(|issue| issue.is_error()) {
        bail!("build aborted because validation failed");
    }

    let view = workspace.view(drafts);
    let out_dir = workspace.config.site_dir(root);
    let staging_dir = out_dir.with_file_name("_site.staging");

    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir).with_context(|| {
            format!(
                "failed to remove stale staging dir {}",
                staging_dir.display()
            )
        })?;
    }

    let build_start = Instant::now();
    let report = build_site_from_view(&view, &staging_dir)?;
    info!(
        page_count = report.page_count,
        topic_count = report.topic_count,
        tag_count = report.tag_count,
        elapsed_ms = build_start.elapsed().as_millis(),
        "built static site"
    );

    let assets_dir = staging_dir.join(&view.config().paths.assets_dir);
    match bundle_browser_search(root, &view.config().search.wasm_module, &assets_dir) {
        Ok(bundle_report) => {
            info!(
                cache_hit = bundle_report.cache_hit,
                asset_bytes = bundle_report.asset_bytes,
                "browser bundle ready"
            );
        }
        Err(err) => {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(err);
        }
    }

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)
            .with_context(|| format!("failed to remove old output dir {}", out_dir.display()))?;
    }
    if let Some(parent) = out_dir.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::rename(&staging_dir, &out_dir).with_context(|| {
        format!(
            "failed to promote staging dir {} to {}",
            staging_dir.display(),
            out_dir.display()
        )
    })?;

    Ok(report)
}
