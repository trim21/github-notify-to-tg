FROM rust:1.97.0-slim-bullseye@sha256:b785e50798a0e85ef3c0e856a401af5029a35531bc93652f3d572836dbed519b AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:bc0f6c3bce611a0bb6784a63e4afc76816b341c9ac36d9322d7b9e5077d24d96
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
