FROM rust:latest AS builder
WORKDIR /app

COPY ./Cargo.toml ./Cargo.lock ./
COPY src/ ./src/
COPY templates/ ./templates/
COPY public/ ./public/
COPY migrations/ ./migrations/

ENV SQLX_OFFLINE=true
RUN cargo build --release
RUN mv ./target/release/whistbook ./app

FROM debian:stable-slim AS runtime
RUN apt-get update && apt-get install -y libssl3 libssl-dev
WORKDIR /app
COPY .env ./.env
COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]
