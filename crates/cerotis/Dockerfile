# Build Stage
FROM ghcr.io/kratos2377/base:latest AS builder

# Bundle Stage
FROM debian:bullseye-slim
RUN sudo apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean
COPY --from=builder /home/rust/src/target/release/cerotis ./
EXPOSE 9000
CMD ["./cerotis"]
