FROM rust:1.98.0-slim-bullseye@sha256:1e4acf0228e68cd94b5ab7a5e0b1ab48963ed55117d0f3b785fc117ee32adef8 AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:dac90a25ffe03ebaace90cda886b081e61ab6cd77ac47eda51f7bcc15c320765
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
