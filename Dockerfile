FROM ubuntu:20.04

ENV DEBIAN_FRONTEND="noninteractive" 

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
    && rm -rf /var/lib/apt/lists/*

COPY ./rust-toolchain /tmp/rust-toolchain

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- -y --no-modify-path --default-toolchain "$(cat /tmp/rust-toolchain)"

RUN cargo install diesel_cli --no-default-features --features "postgres"
COPY . .
RUN cargo build --release -p indexer-explorer
ENV BINARY ./target/release/indexer-explorer
RUN $BINARY init
RUN sed -i 's/[ ]*"tracked_shards"\:.*/  "tracked_shards"\: \[0\],/' ~/.near/config.json
CMD ["diesel migration run && $BINARY run sync-from-latest"]
