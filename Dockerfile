FROM rust:1.97.0-slim-bullseye@sha256:53dd21ec82802c67efd8eb6776b076509074833240e70d914eeae17b14ccf515 AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:bc0f6c3bce611a0bb6784a63e4afc76816b341c9ac36d9322d7b9e5077d24d96
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
