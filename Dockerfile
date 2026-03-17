FROM rust:1.94.0-slim-bullseye@sha256:2caebcde6774a5b01f6e437e435459f8f3077eb3900de57c2f1444d8888c7afa AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:e1cc90d06703f5dc30ae869fbfce78fce688f21a97efecd226375233a882e62f
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
