FROM rust:1.97.0-slim-bullseye@sha256:fdbadc470df655a7957f731e58d43e250e6a2e6817f6e233f7bbda40311478b7 AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:bc0f6c3bce611a0bb6784a63e4afc76816b341c9ac36d9322d7b9e5077d24d96
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
