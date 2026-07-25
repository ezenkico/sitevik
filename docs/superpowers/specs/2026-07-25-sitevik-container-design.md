# Sitevik Container Design

## Goal

Package Sitevik in the smallest practical runtime container and provide a
Docker Compose setup for manually testing it against a representative static
site.

## Image

The `Dockerfile` uses two stages:

1. A `rust:alpine` builder compiles the locked release build. Alpine's musl
   target produces the statically linked executable required by a scratch
   runtime.
2. A `scratch` runtime contains only the Sitevik executable.

The runtime uses numeric user and group `65532:65532`, exposes port 8080, and
starts `/sitevik` directly. It contains no shell, package manager, CA bundle,
health-check utility, or embedded site files.

The existing release profile supplies optimization, fat LTO, symbol stripping,
one code-generation unit, and abort-on-panic behavior. UPX is intentionally
excluded because its additional build tooling and runtime decompression are not
worth the smaller compressed image.

## Compose Setup

`compose.yaml` builds the local `Dockerfile`, publishes host port 8080 to
container port 8080, and mounts `./test-site` read-only at `/site`.

The service sets:

- `SITEVIK_ROOT=/site`
- `BIND_ADDR=0.0.0.0:8080`
- `SITEVIK_SPA=false`

No test-runner service, TLS, health check, or SPA demonstration is included.

## Static Fixture

The repository contains:

- `test-site/index.html`
- `test-site/about/index.html`
- `test-site/assets/style.css`

The fixture supports manual checks that `/` serves the root page, `/about`
serves the directory index, the stylesheet is served directly, and an unknown
path returns 404.

## Build Context

`.dockerignore` excludes Git metadata, local worktrees, `target`, and other
local build output. Source files, `Cargo.toml`, and `Cargo.lock` remain in the
build context.

## Verification

Verification consists of:

1. Building the image with Docker Compose.
2. Starting the service.
3. Requesting `/`, `/about`, and `/assets/style.css` and expecting 200.
4. Requesting a missing path and expecting 404.
5. Inspecting the final image to confirm it contains only the scratch layer and
   Sitevik binary.

The Compose service is intended for manual HTTP testing, not for running the
Rust test suite.
