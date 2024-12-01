FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src

RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release

COPY .env ./.env
COPY templates/ ./templates/
COPY public/ ./public/

RUN cargo build --release
RUN mv ./target/release/whistbook ./app

FROM debian:stable-slim AS runtime
RUN apt-get update && apt-get install libssl3 libssl-dev
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]
