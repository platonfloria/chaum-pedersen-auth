# syntax=docker/dockerfile:experimental
# -- Base Image --
# Installs application dependencies
FROM rust:slim-buster as builder

ARG VERSION

ENV VERSION=$VERSION

# Install dependencies
RUN apt update \
 && apt install -y pkg-config \
 && rustup component add rustfmt

# Set up application environment
WORKDIR /app
RUN cargo new --bin {{service-name}} \
 && mv ./{{service-name}} ./service
WORKDIR /app/service
COPY ./service/Cargo.toml ./service/Cargo.lock ./service/build.rs ./
RUN cargo build --release \
 && rm -r ./src
COPY ./service/src ./src
RUN rm ./target/release/deps/{{service_name}}* \
 && cargo build --release

# -- Test Image --
# Code to be mounted into /app
FROM builder AS test
WORKDIR /app
ENTRYPOINT ["./service/scripts/entry.sh"]

# -- Production Image --
# Runs the service
FROM debian:buster-slim AS prod
WORKDIR /app
COPY ./service/scripts ./scripts
COPY --from=builder /app/service/target/release/{{service-name}} /app/{{service-name}}
ENTRYPOINT ["./scripts/entry.sh", "app"]
