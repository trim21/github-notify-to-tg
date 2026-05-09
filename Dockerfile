FROM rust:1.95.0-slim-bullseye@sha256:ba808bd2856562820d3e07e46a370fc6f38cfa89da6f4f12872bb708d6757814 AS builder
WORKDIR /src

COPY rust-toolchain.toml ./
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:latest@sha256:56aaf20ab2523a346a67c8e8f8e8dabe447447d0788b82284d14ad79cd5f93cc
WORKDIR /app

COPY --from=builder /src/target/release/github-notify-to-tg /usr/local/bin/github-notify-to-tg

ENTRYPOINT ["/usr/local/bin/github-notify-to-tg"]
