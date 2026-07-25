# Project Documentation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add accurate user documentation and repository-specific instructions for coding agents.

**Architecture:** `README.md` is the public usage reference. Root `AGENTS.md` is the binding implementation and verification guide for agents working anywhere in the repository.

**Tech Stack:** Markdown, Cargo, Actix Web, Docker, and Docker Compose.

## Global Constraints

- Document only current behavior.
- Do not mention future developments.
- Keep Sitevik's intentionally small scope explicit.
- Use environment-variable names, defaults, routes, and commands exactly as implemented.
- Do not change source code, dependencies, Docker configuration, or test fixtures.

---

### Task 1: README And Agent Instructions

**Files:**
- Create: `README.md`
- Create: `AGENTS.md`

**Interfaces:**
- Consumes: `Cargo.toml`, `src/config.rs`, `src/server.rs`, `Dockerfile`, and `compose.yaml`
- Produces: public usage documentation and repository-wide agent instructions

- [ ] **Step 1: Write `README.md`**

Include:

1. A concise description of Sitevik.
2. Routing examples for `/`, `/about`, direct files, optional SPA fallback,
   and dotted missing assets.
3. A configuration table for `SITEVIK_ROOT`, `BIND_ADDR`, and `SITEVIK_SPA`.
4. Local commands:

```bash
cargo run --release
cargo test
```

5. Compose commands:

```bash
docker compose up --build
docker compose down
```

6. Direct image commands that mount a static directory read-only and publish
   port 8080.
7. Security and scope notes covering traversal rejection, trusted symlinks,
   hidden files, plain HTTP, and no directory listings.
8. MIT license reference.

Do not include badges, roadmap text, contribution boilerplate, or marketing
copy.

- [ ] **Step 2: Write root `AGENTS.md`**

Use short imperative sections:

- Project scope
- Routing invariants
- Security invariants
- Rust conventions
- Verification
- Container constraints
- Change discipline

Require:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
docker compose config
```

State that loopback transport tests may require permission in restricted
sandboxes. Require Actix's safe path parsing, focused tests for routing or
security changes, scratch runtime, static musl binary, and numeric non-root
user `65532:65532`.

Prohibit unrelated dependencies, CLI features, TLS, directory listings, broad
refactors, and future-looking documentation.

- [ ] **Step 3: Verify accuracy**

Compare every route, default, environment variable, command, and container
setting against:

```text
Cargo.toml
src/config.rs
src/server.rs
Dockerfile
compose.yaml
```

Run:

```bash
rg -n 'future|later|eventually|roadmap|planned' README.md AGENTS.md
git diff --check
```

Expected: the wording scan and whitespace check produce no output.

- [ ] **Step 4: Commit**

```bash
git add README.md AGENTS.md
git commit -m "docs: add project guidance"
```
