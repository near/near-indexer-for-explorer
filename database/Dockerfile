# Builder stage for Diesel CLI
FROM rust:1.70.0 AS builder
RUN cargo install diesel_cli --no-default-features --features postgres

# Final stage with Postgres and Diesel
FROM postgres:14
ENV POSTGRES_USER=postgres
ENV POSTGRES_PASSWORD=postgres
ENV POSTGRES_DB=postgres
ENV DATABASE_URL=postgres://postgres:postgres@:5432/postgres

# Copy Diesel CLI and migration scripts
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin
COPY ./migrations /migrations
COPY wait-for-postgres.sh /docker-entrypoint-initdb.d/

# Set script permissions
RUN chmod +x /docker-entrypoint-initdb.d/wait-for-postgres.sh
