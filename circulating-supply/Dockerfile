FROM rust:1.64 AS builder
WORKDIR /tmp/
COPY Cargo.toml Cargo.lock ./
COPY database database
# Copy 'indexer' because is member of the workspace but is not compiled or added to the
# output image.
COPY indexer indexer
COPY circulating-supply circulating-supply
RUN cargo build -p circulating-supply --release

FROM ubuntu:20.04
RUN apt update && apt install -yy openssl ca-certificates
RUN apt-get install libpq5 -y
COPY --from=builder /tmp/target/release/circulating-supply .
ENTRYPOINT [ "./circulating-supply" ]
