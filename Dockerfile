FROM rust:slim-bullseye as builder
WORKDIR /app
RUN USER=root cargo new backend-dogfight-rust-24-q1
WORKDIR /app/backend-dogfight-rust-24-q1
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm -rf src

COPY src src

RUN touch src/main.rs
RUN cargo build --release


FROM debian:bullseye-slim
COPY --from=builder /app/backend-dogfight-rust-24-q1/target/release/backend-dogfight-rust-24-q1 /usr/local/bin/

CMD ["backend-dogfight-rust-24-q1"]