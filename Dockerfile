FROM rustlang/rust:nightly-bookworm

# dummy file for dependency building
RUN mkdir src
RUN echo 'fn main() { panic!("Dummy Image Called!")}' > ./src/main.rs
COPY ["Cargo.toml", "Cargo.lock",  "./"]
RUN cargo build --release

COPY src src
#need to break the cargo cache
RUN touch ./src/main.rs

COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY .env ./.env

COPY templates/ ./templates/
COPY public/ ./public/

RUN cargo build --release

ENV RUST_LOG=debug
