# Use the official Rust image.
# https://hub.docker.com/_/rust
FROM rust AS builder

# Copy local code to the container image.
WORKDIR /usr/src/rsky
COPY Cargo.toml Cargo.lock rust-toolchain ./
COPY rsky-lexicon rsky-lexicon
COPY rsky-common rsky-common
COPY rsky-labeler rsky-labeler
COPY rsky-identity rsky-identity
COPY rsky-syntax rsky-syntax
COPY rsky-pds/Cargo.toml rsky-pds/Cargo.toml
COPY rsky-crypto rsky-crypto
COPY rsky-feedgen rsky-feedgen
COPY rsky-firehose rsky-firehose

# Create an empty src directory to trick Cargo into thinking it's a valid Rust project
RUN mkdir rsky-pds/src && echo "fn main() {}" > rsky-pds/src/main.rs

## Install production dependencies and build a release artifact.
RUN cargo build --release --package rsky-pds

COPY rsky-pds/src rsky-pds/src

RUN cargo build --release --package rsky-pds


FROM rust

WORKDIR /usr/src/rsky

COPY --from=builder /usr/src/rsky/target/release/rsky-pds rsky-pds

CMD ["./rsky-pds"]

## Use the official Rust image.
## https://hub.docker.com/_/rust
#FROM rust
#
## Copy local code to the container image.
#WORKDIR /usr/src/rsky
#COPY . .
#
## Install production dependencies and build a release artifact.
#RUN cargo build --package rsky-pds
#
## Run the web service on container startup.
#CMD ["sh", "-c", "ROCKET_PORT=$PORT ROCKET_ADDRESS=0.0.0.0 cargo run --package rsky-pds"]