FROM rust:1.79.0-alpine3.20 AS build-stage

WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev build-base

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs  && cargo build --release  && rm -r ./target/release/deps/* && rm src/main.rs

COPY . .

RUN cargo build --release

FROM alpine:3.20

WORKDIR /app

RUN apk add --no-cache ffmpeg ca-certificates

COPY --from=build-stage /app/target/release/rust-beiboot .

EXPOSE 8080

CMD ["./rust-beiboot"]
