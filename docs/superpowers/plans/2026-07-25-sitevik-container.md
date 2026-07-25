# Sitevik Container Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Package Sitevik as a statically linked executable in a scratch image and provide a Compose-mounted static test site.

**Architecture:** A multi-stage Dockerfile builds Sitevik with musl in Alpine and copies only the release executable into scratch. Compose mounts a small repository fixture read-only and publishes the HTTP service on port 8080.

**Tech Stack:** Docker, Docker Compose, Rust Alpine builder, scratch runtime, HTML, and CSS.

## Global Constraints

- The final runtime image must use `scratch`.
- The runtime image contains only the Sitevik executable.
- The process runs as numeric user and group `65532:65532`.
- `test-site` is mounted read-only at `/site`.
- Compose sets `SITEVIK_ROOT=/site`, `BIND_ADDR=0.0.0.0:8080`, and `SITEVIK_SPA=false`.
- Compose publishes host port 8080 to container port 8080.
- Do not add UPX, TLS, a health check, a test-runner service, or SPA behavior.

---

### Task 1: Scratch Image And Manual Test Site

**Files:**
- Create: `Dockerfile`
- Create: `.dockerignore`
- Create: `compose.yaml`
- Create: `test-site/index.html`
- Create: `test-site/about/index.html`
- Create: `test-site/assets/style.css`

**Interfaces:**
- Produces: Docker image entry point `/sitevik`
- Produces: Compose service `sitevik` at `http://localhost:8080`
- Consumes: repository `Cargo.toml`, `Cargo.lock`, and Rust sources

- [ ] **Step 1: Create the static fixture**

Create a compact HTML root page linking `/assets/style.css` and `/about`.
Create an about page linking back to `/`. Both pages should identify which
route is being served so manual checks are unambiguous.

Create a small stylesheet with readable system-font typography. Do not add
JavaScript, images, frameworks, or SPA routing.

- [ ] **Step 2: Create the multi-stage Dockerfile**

Use this structure:

```dockerfile
FROM rust:alpine AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release

FROM scratch
COPY --from=builder /build/target/release/sitevik /sitevik
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/sitevik"]
```

Do not install runtime packages or copy site content into the image.

- [ ] **Step 3: Limit the Docker build context**

Create `.dockerignore` containing:

```text
.git
.worktrees
.superpowers
target
test-site
docs
```

The source tree and locked Cargo manifests must remain available to the
builder.

- [ ] **Step 4: Create the Compose service**

Create `compose.yaml` with one `sitevik` service:

```yaml
services:
  sitevik:
    build:
      context: .
    ports:
      - "8080:8080"
    environment:
      SITEVIK_ROOT: /site
      BIND_ADDR: 0.0.0.0:8080
      SITEVIK_SPA: "false"
    volumes:
      - ./test-site:/site:ro
```

- [ ] **Step 5: Validate configuration syntax**

Run:

```bash
docker compose config
```

Expected: exit 0 and one resolved `sitevik` service.

- [ ] **Step 6: Build the image**

Run:

```bash
docker compose build
```

Expected: the musl release build succeeds and the scratch runtime image is
created.

- [ ] **Step 7: Run manual HTTP smoke checks**

Start the service:

```bash
docker compose up -d
```

Verify:

```bash
curl --fail http://localhost:8080/
curl --fail http://localhost:8080/about
curl --fail http://localhost:8080/assets/style.css
curl --silent --output /dev/null --write-out '%{http_code}\n' http://localhost:8080/missing
```

Expected: the first three commands succeed with fixture content; the last
prints `404`.

Stop the service:

```bash
docker compose down
```

- [ ] **Step 8: Verify the runtime image and repository**

Run:

```bash
docker image inspect sitevik-sitevik
git diff --check
git status --short
```

Expected: image configuration shows `/sitevik` as entry point and numeric
non-root user `65532:65532`; no whitespace errors are reported; only the six
planned files are uncommitted.

- [ ] **Step 9: Commit**

```bash
git add Dockerfile .dockerignore compose.yaml test-site
git commit -m "feat: add scratch container setup"
```
