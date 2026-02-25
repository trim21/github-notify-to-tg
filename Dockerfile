FROM rust:1.93.1-slim-bullseye@sha256:6137a7b1d4ff24ef2fe63e6ae40ed9b3d17b48f70fd2516a9c52c9fa54ea2391 AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:22fd4bd55e5f0ef1929985f111816ba1e43c00a0ddeb001c0fdfb2724b4e3cc2
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
