FROM rust:1.89-bookworm AS builder

RUN apt-get update

WORKDIR /app
COPY . .
RUN rm -rf target
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:latest AS runner

COPY --from=builder /app/target/release/rinha /bin/

CMD ["/bin/rinha"]
