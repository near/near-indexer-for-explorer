# syntax=docker/dockerfile-upstream:experimental

FROM ubuntu:18.04 as build

RUN apt-get update -qq && apt-get install -y \
    git \
    cmake \
    g++ \
    pkg-config \
    libssl-dev \
    curl \
    llvm \
    clang \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY ./rust-toolchain /tmp/rust-toolchain

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- -y --no-modify-path --default-toolchain "$(cat /tmp/rust-toolchain)"

RUN cargo install diesel_cli --no-default-features --features "postgres" --bin diesel

WORKDIR /near
RUN cargo +"$(cat /tmp/rust-toolchain)" new --bin indexer-explorer
WORKDIR /near/indexer-explorer

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# Build only dependencies first (which take ~45 minutes), so that they'll be cached for as long as the Cargo.toml/.lock aren't changed
ENV CARGO_TARGET_DIR=/tmp/target
ENV RUSTC_FLAGS='-C target-cpu=x86-64'
ENV PORTABLE=ON
RUN cargo +"$(cat /tmp/rust-toolchain)" build --release
RUN rm src/*.rs
RUN rm /tmp/target/release/indexer-explorer*

COPY . .

# This touch is necessary so that Rust doesn't skip the build (even though the source has completely changed... Rust cache is weird :P)
RUN touch src/main.rs

RUN cargo +"$(cat /tmp/rust-toolchain)" build --release -p indexer-explorer

# If the --store-genesis flag isn't set, the accounts in genesis won't get created in the DB which will lead to foreign key constraint violations
# See https://github.com/near/near-indexer-for-explorer/issues/167
CMD diesel migration run && \
    /tmp/target/release/indexer-explorer --home-dir /root/.near/localnet init ${BOOT_NODES:+--boot-nodes=${BOOT_NODES}} --fast --chain-id localnet && \
    sed -i 's/"tracked_shards": \[\]/"tracked_shards": \[0\]/' /root/.near/localnet/config.json && \
    /tmp/target/release/indexer-explorer --home-dir /root/.near/localnet run --store-genesis sync-from-latest
