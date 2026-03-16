FROM rust:1.94.0-slim-bullseye@sha256:34de77eca70eb7c1a4a8b4ccbe2dbfdd433594c1ff5a45102a1e61229b6fa940 AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:e1cc90d06703f5dc30ae869fbfce78fce688f21a97efecd226375233a882e62f
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
