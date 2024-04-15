# Build Stage
FROM ghcr.io/vortex/base:latest AS builder

# Bundle Stage
FROM debian:bullseye-slim
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean
COPY --from=builder /home/rust/src/target/release/messier ./
EXPOSE 3004
CMD ["./messier"]
