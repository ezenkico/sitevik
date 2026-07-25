# Project Documentation Design

## Goal

Add concise documentation for Sitevik users, contributors, and coding agents.
The documentation describes the current project only and does not discuss
future development.

## README

`README.md` will document:

- Sitevik's purpose and routing behavior.
- Configuration variables and defaults.
- Local Cargo build and run commands.
- Docker Compose usage with the included static test site.
- Direct Docker image build and run commands.
- Path-security behavior and the intentionally small feature scope.
- The MIT license.

Examples will use the existing names and defaults exactly.

## Agent Instructions

The root `AGENTS.md` will define:

- The minimal product scope and behaviors that must remain stable.
- Rust and Actix implementation conventions.
- Static routing, SPA asset, and traversal-security invariants.
- Required format, lint, test, and release-build commands.
- Scratch-container, static-linking, and non-root runtime requirements.
- Rules against unrelated dependencies, CLI features, TLS, and broad
  refactoring.

The instructions will be direct and repository-specific. They will not repeat
generic software-engineering advice or mention potential future features.

## Verification

Review both files against the current source, Compose configuration, and
Dockerfile. Check Markdown formatting, command accuracy, environment-variable
names, defaults, and absence of future-looking language.
