FROM rust:1.85-alpine AS builder
WORKDIR /src

RUN apk add --no-cache musl-dev

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

FROM alpine:3.21
RUN adduser -D -h /app appuser
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

USER appuser
ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
