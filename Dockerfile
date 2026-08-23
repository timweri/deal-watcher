FROM rust:1.97-slim AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/deal-watcher /usr/local/bin/deal-watcher

# Default port for /healthz; if you override HEALTHZ_PORT at runtime, update
# your published port mapping accordingly (EXPOSE is fixed at build time).
EXPOSE 8080

CMD [ "deal-watcher" ]
