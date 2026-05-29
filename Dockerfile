FROM rust:1.95-bookworm AS builder

WORKDIR /app

ENV DATABASE_URL=sqlite:////app/database.db

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
RUN sqlite3 /app/database.db < migrations/20260524104134_urls.sql
RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/makise-kurisu-bot /usr/local/bin/makise-kurisu-bot

ENV RUST_LOG=info

CMD ["/usr/local/bin/makise-kurisu-bot"]
