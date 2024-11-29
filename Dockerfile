# 1. Install chef & sccache & protobuf compiler
FROM rust:1.82 AS base
WORKDIR /build
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall -y cargo-chef sccache
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

# 1.1 Install protobuf compiler & supervisor
RUN apt-get update && apt-get install -y protobuf-compiler libprotobuf-dev supervisor

# 2. Prepare recipe file
FROM base AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 3. Build changed dependencies
FROM base AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --recipe-path recipe.json --release

COPY . .

# 3.1 Build indexer
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build -p sparker-indexer --release

# 3.2 Build api
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build -p sparker-api --release

# 4. Runtime
FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /build/target/release/sparker-indexer .
COPY --from=builder /build/target/release/sparker-api .
COPY ./config.mainnet.json ./config.mainnet.json

COPY ./supervisord.conf /etc/supervisord.conf

EXPOSE 50051 3011

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisord.conf"]
