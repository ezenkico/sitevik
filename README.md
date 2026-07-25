# Sitevik

Sitevik is a minimal Rust static-site server built with Actix Web.

It serves files from one directory, resolves directory indexes, and can
optionally fall back to the root `index.html` for single-page applications.

## Routing

Given this site:

```text
index.html
about/index.html
assets/style.css
```

Sitevik resolves requests as follows:

| Request | Response |
| --- | --- |
| `/` | `/index.html` |
| `/about` | `/about/index.html` |
| `/about/` | `/about/index.html` |
| `/assets/style.css` | The file itself |

When SPA mode is enabled, a missing extensionless route such as `/dashboard`
serves `/index.html`. A missing final segment containing `.`, such as
`/assets/app.js`, always returns 404.

## Configuration

Sitevik is configured only through environment variables.

| Variable | Default | Description |
| --- | --- | --- |
| `SITEVIK_ROOT` | `./dist` | Directory containing the static site. |
| `BIND_ADDR` | `0.0.0.0:8080` | Socket address to listen on. |
| `SITEVIK_SPA` | `false` | Enables SPA fallback when set to `true`. |

`SITEVIK_SPA` accepts only `true` or `false`. Invalid configuration stops the
server at startup.

## Run Locally

Rust 2024 edition support is required.

```bash
SITEVIK_ROOT=./test-site cargo run --release
```

The server is available at <http://localhost:8080>.

Run the test suite with:

```bash
cargo test
```

## Docker Compose

The included Compose service builds the scratch image and mounts `test-site`
read-only:

```bash
docker compose up --build
```

Open <http://localhost:8080> and <http://localhost:8080/about>. Stop the
service with:

```bash
docker compose down
```

SPA fallback is disabled in the Compose setup.

## Docker

Build the image:

```bash
docker build -t sitevik .
```

Run it with a static directory mounted at `/site`:

```bash
docker run --rm \
  -p 8080:8080 \
  -e SITEVIK_ROOT=/site \
  -v "$PWD/test-site:/site:ro" \
  sitevik
```

The runtime image is based on `scratch`, contains only the statically linked
Sitevik executable, and runs as numeric user and group `65532:65532`.

## Security And Scope

- URL traversal and malformed paths return 404.
- Directory listings are disabled.
- Hidden files under the configured root are served.
- The content directory is trusted input. Normal operating-system symlink
  resolution applies.
- Only `GET` and `HEAD` serve content. Other methods return 405.
- Sitevik serves plain HTTP. TLS and public-edge policy belong at a reverse
  proxy.
- Sitevik has no command-line configuration.

## License

Sitevik is available under the [MIT License](LICENSE).
