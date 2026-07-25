# Sitevik Design

## Purpose

Sitevik is a deliberately small Rust HTTP service for serving a prebuilt static
site. It supports directory indexes and an optional single-page application
(SPA) fallback without adding TLS, a command-line interface, or general web
server configuration.

## Dependencies

The key libraries are:

- `actix-web` for the HTTP server and application lifecycle.
- `actix-files` for secure static-path handling and file responses.

Sitevik delegates path validation, MIME types, range requests, and standard
file-response metadata to `actix-files` rather than reimplementing them.

## Configuration

Sitevik uses environment variables only:

| Variable | Default | Behavior |
| --- | --- | --- |
| `SITEVIK_ROOT` | `./dist` | Directory containing the site output. |
| `BIND_ADDR` | `0.0.0.0:8080` | Socket address on which the server listens. |
| `SITEVIK_SPA` | `false` | Strict boolean controlling SPA fallback. |

`SITEVIK_SPA` accepts only `true`, `false`, or an unset value. Any other value
causes startup to fail with a concise error.

Startup also fails when `SITEVIK_ROOT` is not a readable directory, when
`BIND_ADDR` is invalid, or when the address cannot be bound. The root
`index.html` is not required at startup; requests that need an absent index
receive a 404 response.

## Architecture

Sitevik is one binary with two small internal responsibilities:

1. Load and validate configuration.
2. Build and run an Actix application containing one root-mounted
   `actix_files::Files` service and a narrow fallback handler.

The file service enables directory indexes with `index.html`, permits hidden
files, keeps directory listings disabled, and does not redirect directory
requests to slash-ended paths. The fallback handler applies only the SPA and
missing-asset rules after normal static-file resolution fails.

There is no CLI or additional configuration layer.

## Request Resolution

Sitevik supports `GET` and `HEAD`. Other methods return
`405 Method Not Allowed`.

For each supported request:

1. Actix decodes and validates the URL path using its secure static-path
   handling. Parent-directory components are rejected.
2. If the path names an existing file, Sitevik serves it directly.
3. If the path names an existing directory containing `index.html`, Sitevik
   serves that index without redirecting.
4. If no static file matched, SPA mode is enabled, and the final decoded path
   segment does not contain `.`, Sitevik serves the root `index.html`.
5. Every other missing path returns 404.

Consequently:

- `/` serves `/index.html`.
- `/about` and `/about/` both serve `/about/index.html`.
- Existing files are served directly.
- A missing path such as `/dashboard` can receive the SPA fallback.
- A missing path such as `/app.js` never receives the SPA fallback.

Only the final path segment determines whether a missing request is an asset.
Query strings do not participate in filesystem resolution or asset
classification.

## Security Model

Literal and percent-encoded traversal attempts, including `/../secrets` and
`/%2e%2e/secrets`, must not resolve to filesystem paths and return 404.
Malformed or otherwise invalid paths also return 404 without exposing
filesystem details.

The configured content directory is trusted deployment input. Sitevik adds no
special symlink policy; normal operating-system symlink resolution applies.
Operators are responsible for the content tree and its symlinks.

Hidden files and hidden directories below the configured content root are
eligible for normal serving. Directory listings remain disabled.

## Error Handling

Responses do not expose local filesystem paths:

- Missing files, directories without an index, invalid paths, and traversal
  attempts return 404.
- Unsupported methods return 405.
- Unexpected filesystem read failures return 500.
- A missing root `index.html` produces 404 for `/` and for requests that would
  otherwise use the SPA fallback.

Configuration and bind errors are reported at startup and terminate the
process with a failure status.

## Release Profile

Release builds prioritize runtime speed while applying size reductions that do
not replace speed optimization:

- `opt-level = 3`
- fat link-time optimization
- one code-generation unit
- stripped symbols
- abort-on-panic behavior

These settings intentionally trade slower release compilation and reduced
panic diagnostics for runtime optimization and a smaller deployable binary.

## Testing

Focused automated tests cover:

- Configuration defaults and strict `SITEVIK_SPA` parsing.
- Invalid roots and bind addresses.
- Direct file serving and MIME behavior.
- `/`, `/about`, and `/about/` directory-index behavior.
- SPA fallback when enabled and disabled.
- A missing dotted final segment returning 404 with SPA mode enabled.
- Hidden-file serving and disabled directory listings.
- Literal and percent-encoded traversal attempts.
- `GET`, `HEAD`, and rejected HTTP methods.
- Missing root indexes and unexpected filesystem failures.
- The required Cargo release-profile settings.

Tests should use temporary content directories and Actix's in-process test
support. Benchmarks and browser-driven tests are outside this version's scope.

## Explicit Non-Goals

This version does not include:

- TLS or certificate management.
- Reverse-proxy or forwarding-header interpretation.
- CLI arguments or subcommands.
- Directory listings.
- File watching or live reload.
- Compression configuration.
- Custom cache policy.
- Custom error pages.
- Authentication or access control.

TLS and broader deployment features may be considered for a separate, larger
version rather than expanding this minimal server.
