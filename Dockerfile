FROM rust:1.93.1-slim-bullseye@sha256:6137a7b1d4ff24ef2fe63e6ae40ed9b3d17b48f70fd2516a9c52c9fa54ea2391 AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:8c1a496b055d36c222e95ebd5c53bdd1fee447689a97ad889febca8380d578ec
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
