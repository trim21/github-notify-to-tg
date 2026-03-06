FROM rust:1.94.0-slim-bullseye@sha256:206ebe094d139aac6a046b3b6cbac6e39da0ab465fd15b359d958c6bbf969acd AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:8c1a496b055d36c222e95ebd5c53bdd1fee447689a97ad889febca8380d578ec
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
