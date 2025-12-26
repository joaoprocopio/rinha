FROM rust:1.92 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:12-slim AS runner

COPY --from=builder /app/target/release/server /app/target/release/worker /app/
COPY entrypoint.sh /app/
RUN chmod +x /app/entrypoint.sh

CMD ["/app/entrypoint.sh"]
