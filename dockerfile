FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake

WORKDIR /app
COPY . .
RUN rm -rf target
RUN cargo build --release

FROM debian:bookworm-slim AS runner

COPY --from=builder /app/target/release/rinha /bin
RUN chmod +x /bin/rinha

CMD ["rinha"]