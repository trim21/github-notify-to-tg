FROM rust:1.93.1-slim-bullseye@sha256:eca54108db942b7003a5753cae5588004d0f06e9df8a9bad9e28af17dbd8a8ea AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
RUN cargo fetch

COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:22fd4bd55e5f0ef1929985f111816ba1e43c00a0ddeb001c0fdfb2724b4e3cc2
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
