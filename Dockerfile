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
