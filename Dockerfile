FROM rust:1-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/sipp /usr/local/bin/sipp
WORKDIR /data
EXPOSE 3000
CMD ["sipp", "server", "--port", "3000", "--host", "0.0.0.0"]
