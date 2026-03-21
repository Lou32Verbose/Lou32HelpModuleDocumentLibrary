# Contributing

## First-time Setup

```powershell
# install Rust via rustup (https://rustup.rs)
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

Then run the verification sequence to confirm everything works:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -- check
cargo run -- build
pwsh -File scripts/run-audit.ps1
```

Set `RUST_LOG=info` to see validation counts, build timing, browser bundle cache decisions, and search index size logs.

## Creating New Content

Use the `new` command to scaffold a correctly structured document:

```powershell
cargo run -- new powershell/networking dns-troubleshooting --type recipe
```

This creates a Markdown file with the proper frontmatter fields and required section headings for the given page type.

## Page Types

| Type | When to use | Required sections |
|------|-------------|-------------------|
| `reference` | Describes a tool, command, or concept | Synopsis, Syntax, Parameters/Flags, Examples, Related |
| `recipe` | Step-by-step task guide | Goal, Prerequisites, Steps, Commands, Verification, Related |
| `troubleshooting` | Diagnosis and fix guide | Symptoms, Cause, Resolution, Verification, Related |
| `template` | Reusable template with placeholders | Use Case, Template, Variables, Examples, Related |

## Frontmatter Schema

Every Markdown page under `content/` must include YAML frontmatter with these fields (in order):

| Field | Type | Rules |
|-------|------|-------|
| `title` | string | Human-readable page title |
| `slug` | string | Must start and end with `/`, begin with topic path |
| `summary` | string | One-line description for search results and index pages |
| `topic` | string | Topic path matching a `[[topics]]` key in `lou32help.toml` |
| `type` | string | One of: `reference`, `recipe`, `troubleshooting`, `template` |
| `tags` | string[] | Lowercase, hyphenated; each becomes a `/tags/<tag>/` route |
| `aliases` | string[] | Alternative lookup names; must be globally unique |
| `platforms` | string[] | Target platforms (e.g. `windows`, `powershell`, `linux`) |
| `related` | string[] | Slugs of related documents (must resolve to existing pages) |
| `status` | string | `published` or `draft` |
| `updated` | date | Last updated date (YYYY-MM-DD) |

## Content Authoring Rules

- Slugs and related slugs must start and end with `/` and must not contain empty segments, `.`, `..`, backslashes, drive letters, NULs, Windows-reserved filename characters, or trailing dots/spaces.
- Topic paths must not start or end with `/` and must follow the same safe-segment rules.
- Tags become `/tags/<tag>/` routes, so they must be a single safe segment.
- Avoid Windows reserved names such as `con`, `prn`, `aux`, `nul`, `com1`, or `lpt1` in any route-affecting metadata.

## PR Process

Before submitting changes:

1. Run the full verification sequence (see First-time Setup above)
2. All validation codes must be clean: `cargo run -- check`
3. The build must succeed: `cargo run -- build`
4. New content should render correctly: `cargo run -- serve --port 4000`

## Coding Conventions

- Comments explain "why" not "what"
- Public items require `///` doc comments (`#![warn(missing_docs)]` is enabled in `lou32help-core` and `lou32help-site`)
- New validation error codes must be added to the catalog comment in `crates/lou32help-core/src/validation.rs`
- Run `cargo fmt` before committing; `cargo clippy` must pass with no warnings

## Build Notes

- Browser search assets are cached under `target/lou32help/browser-search/`.
- Warm builds reuse cached browser assets when the bundle fingerprint is unchanged.
- If browser bundling fails, confirm `wasm32-unknown-unknown` is installed via `rustup` and `wasm-bindgen-cli` is available on `PATH`.
