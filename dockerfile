FROM rust:1.88-bookworm AS builder

WORKDIR /app
COPY . .
RUN rm -rf target
RUN cargo build --release --bin rinha

FROM debian:bookworm-slim AS runner

COPY --from=builder /app/target/release/rinha /bin
RUN chmod +x /bin/rinha

CMD ["rinha"]