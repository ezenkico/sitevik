# Sitevik Agent Instructions

## Project Scope

- Keep Sitevik a minimal static-site server.
- Configure it only through `SITEVIK_ROOT`, `BIND_ADDR`, and `SITEVIK_SPA`.
- Do not add CLI features, TLS, directory listings, authentication, proxy
  handling, file watching, or unrelated dependencies.
- Keep documentation limited to current behavior.

## Routing Invariants

- `/` serves the root `index.html`.
- `/path` and `/path/` serve `/path/index.html` when it exists.
- Existing files are served directly.
- SPA fallback is optional and serves the root `index.html` only for missing
  routes whose final segment contains no `.`.
- Missing dotted assets return 404 even when SPA fallback is enabled.
- Only `GET` and `HEAD` serve files. Other methods return 405.

## Security Invariants

- Reject literal, encoded, malformed, and dot-segment traversal before joining
  a request path to the content root.
- Preserve Actix's safe path parsing. Do not replace it with unchecked string
  concatenation.
- Invalid paths return empty 404 responses without local filesystem details.
- Unexpected filesystem failures return empty 500 responses.
- Treat the configured content root and its symlinks as trusted operator input.
- Keep directory listings disabled.

## Rust Conventions

- Use stable Rust with edition 2024.
- Prefer existing Actix Web and `actix-files` primitives.
- Keep configuration parsing in `src/config.rs`, file-serving behavior in
  `src/server.rs`, and process startup in `src/main.rs`.
- Keep changes small and avoid unrelated refactors.
- Add focused regression tests for every routing, fallback, or path-security
  behavior change.
- Do not weaken the release profile in `Cargo.toml`.

## Verification

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
docker compose config
```

The raw HTTP transport tests bind ephemeral loopback ports and can require
additional permission in restricted sandboxes.

## Container Constraints

- Build the release executable with musl.
- Keep the runtime stage based on `scratch`.
- Copy only `/sitevik` into the runtime image.
- Run as numeric user and group `65532:65532`.
- Keep the test site outside the image and mount it read-only.
- Do not add a shell, package manager, CA bundle, health-check utility, or UPX.

## Change Discipline

- Do not revert user changes or modify unrelated files.
- Do not add abstractions unless they remove concrete complexity.
- Keep dependency additions justified by required behavior.
- Match tests to the risk of the change.
- Keep README examples synchronized with executable and Compose behavior.
