# Operations

## Production Build Prerequisites

- Rust toolchain with the workspace `rust-version`
- `wasm32-unknown-unknown` target installed
- `wasm-bindgen-cli` installed and available on `PATH`

## Standard Verification Sequence

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p lou32help-cli -- check
cargo run -p lou32help-cli -- build
pwsh -File scripts/run-audit.ps1
```

## Browser Bundle Recovery

- If the build reports a missing wasm target, run `rustup target add wasm32-unknown-unknown`.
- If the build reports a missing `wasm-bindgen`, run `cargo install wasm-bindgen-cli`.
- If cached browser assets appear stale or corrupted, delete `target/lou32help/browser-search/` and rebuild.
- The browser bundle cache is disposable and should never be checked into source control.

## Staging Directory Cleanup

The build creates a `_site.staging` directory next to the output directory and atomically renames it on success. If a build is interrupted (e.g. by Ctrl-C or a file lock from a running preview server), the stale staging directory can be safely deleted:

```powershell
Remove-Item -Recurse -Force dist/_site.staging
```

## General Troubleshooting

- If `cargo run -- check` reports validation errors, each error includes a code (e.g. `duplicate-alias`, `missing-section`). See the error code catalog in `crates/lou32help-core/src/validation.rs` for descriptions.
- If `lou32help-web-search` fails to compile, ensure `lou32help-core` builds cleanly first: `cargo check -p lou32help-core`.
- If `Cargo.lock` is stale after dependency changes, run `cargo update` and re-test.
- Set `RUST_LOG=debug` for detailed timing and cache-decision output during builds.

## CI Guidance

A minimal CI pipeline should run the Standard Verification Sequence above. The browser bundle step requires the `wasm32-unknown-unknown` target and `wasm-bindgen-cli` to be pre-installed in the CI image.

## Deployment

The output in `dist/site/` is a self-contained static site. All internal links are relative, so it can be deployed to any web server path or opened from the local filesystem. The web server must serve `.wasm` files with the `application/wasm` MIME type for browser search to work.

## Release Expectations

- `cargo run -p lou32help-cli -- check` must be clean against repo content before release.
- A successful build must emit `dist/site/assets/search-index.json`, `<module>.js`, and `<module>_bg.wasm`.
- Browser search should load without console errors and route-safe validation should reject unsafe metadata before content ships.
