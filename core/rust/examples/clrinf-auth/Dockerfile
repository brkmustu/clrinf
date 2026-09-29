FROM rust:alpine AS builder
WORKDIR /src

RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./
COPY crates/clrinf-core ./crates/clrinf-core
COPY crates/clrinf-adapters ./crates/clrinf-adapters
COPY examples/clrinf-auth ./examples/clrinf-auth

RUN cargo build --release --bin clrinf-auth --locked

FROM alpine:3.19
WORKDIR /app
RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /src/target/release/clrinf-auth /app/clrinf-auth
COPY examples/clrinf-auth/politikalar /app/politikalar

ENV PORT=8081
EXPOSE 8081

ENTRYPOINT ["/app/clrinf-auth"]
