# Architecture

## Workspace Layout

- `crates/lou32help-core`: document parsing, normalization, validation, search indexing, and workspace projection
- `crates/lou32help-site`: static HTML rendering, output-path enforcement, and browser search asset emission
- `crates/lou32help-cli`: command dispatch, validation/build orchestration, preview serving, and cached browser bundling
- `crates/lou32help-web-search`: WASM adapter that runs browser-side search against the browser search index

## Crate Dependency Graph

```
lou32help-cli ──→ lou32help-site ──→ lou32help-core
     │                                     ↑
     └─────────────────────────────────────┘
lou32help-web-search ──→ lou32help-core
```

`lou32help-core` is the shared foundation (parsing, validation, search, rendering). `lou32help-site` adds static HTML generation via `maud` compile-time templates. `lou32help-cli` orchestrates both and invokes `lou32help-web-search` indirectly via `cargo build` during browser bundling.

## WorkspaceView

`Workspace` holds the full parsed document set. `WorkspaceView<'a>` is a borrow-based projection created via `workspace.view(include_drafts)` that pre-filters by publication status and pre-builds slug/alias indexes, topic trees, tag indexes, and related-document graphs. Both CLI display functions and the site builder accept a `WorkspaceView`.

## Validation Flow

1. `Document::from_file` parses frontmatter and records raw route/path issues before normalization.
2. `Workspace::validate` combines document issues, config topic-key checks, schema/content checks, and link validation.
3. `build` and `serve` call validation before site generation and stop on any error-level issue.
4. `lou32help-site` performs a second output-path safety check before writing any dynamic page.

## Search Flow

- `lou32help-core` builds a full in-memory `SearchIndex` for CLI lookups.
- `lou32help-site` serializes a dedicated `BrowserSearchIndex` with bounded body text into `dist/site/assets/search-index.json`.
- `lou32help-web-search` deserializes the browser index in WASM and applies the same field-aware ranking model used by the core search implementation where those fields still exist.

## HTML Templating

`lou32help-site` uses the `maud` crate for compile-time HTML generation. Page render functions return `maud::Markup` values converted to strings via `.into_string()`. The browser search script is generated from `search.js.tpl`, a JavaScript template with placeholders `__WASM_MODULE__`, `__MIN_QUERY_LENGTH__`, `__MAX_RESULTS__`, and `__WASM_INTEGRITY__` that are replaced at build time with values from `lou32help.toml` and the WASM bundle.

## URL Rewriting

All page templates emit root-relative URLs (`href="/topics/..."`, `src="/assets/..."`). Before writing output files, `finalize_html()` rewrites these to relative paths using `rewrite_root_relative_urls()`. This enables the static site to be served from any base path or opened directly from the filesystem.

## Browser Bundle Pipeline

1. `lou32help-site` writes `search.js` with the integrity placeholder and the browser index JSON.
2. `lou32help-cli` fingerprints the bundle inputs and checks `target/lou32help/browser-search/`.
3. On a cache miss, the CLI verifies the wasm target and `wasm-bindgen`, builds `lou32help-web-search`, and stores generated JS/WASM plus manifest metadata under `target/`.
4. On a cache hit, the CLI copies the cached JS/WASM into `dist/site/assets/` and injects the cached SRI hash into `search.js`.
